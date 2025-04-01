#![allow(static_mut_refs)]
#![feature(inherent_str_constructors)]

use std::path::PathBuf;

use libafl_bolts::tuples::tuple_list;
use serde_json;
use json;
use libafl::{
    corpus::{CachedOnDiskCorpus, Corpus, OnDiskCorpus},
    events::SimpleEventManager,
    executors::{inprocess::InProcessExecutor, ExitKind},
    feedback_or, feedback_or_fast,
    feedbacks::{CrashFeedback, MaxMapFeedback, TimeFeedback, TimeoutFeedback, AflMapFeedback}, fuzzer::{Fuzzer, StdFuzzer}, inputs::{BytesInput, HasTargetBytes}, monitors::{MultiMonitor, SimpleMonitor}, mutators::{havoc_mutations, StdScheduledMutator}, observers::{HitcountsMapObserver, StdMapObserver, TimeObserver}, prelude::CanTrack, schedulers::QueueScheduler, stages::mutational::StdMutationalStage, state::{HasCorpus, StdState}
};

use libafl_targets::coverage::EDGES_MAP;
use libafl_targets::sancov_pcguard;


fn main() {
    let mon = MultiMonitor::new(|s| println!("{s}"));
    let mut mgr = SimpleEventManager::new(mon);
    let edges_observer =
        HitcountsMapObserver::new(unsafe { StdMapObserver::new("edges", &mut EDGES_MAP) });

    let time_observer = TimeObserver::new("time");

    let map_feedback = AflMapFeedback::new(&edges_observer);
    let mut feedback = feedback_or!(
        map_feedback,
        TimeFeedback::new(&time_observer)
    );

    let mut objective =
        feedback_or_fast!(CrashFeedback::new(), TimeoutFeedback::new());

    let mut state = StdState::new(
        libafl_bolts::rands::StdRand::with_seed(0),
        CachedOnDiskCorpus::<BytesInput>::new("./corpus", 1024).unwrap(),
        OnDiskCorpus::new(PathBuf::from("./crashes")).unwrap(),
        &mut feedback,
        &mut objective,
    ).unwrap();

    let scheduler = QueueScheduler::new();

    let mut fuzzer = StdFuzzer::new(scheduler, feedback, objective);

    let mut harness = |input: &BytesInput| -> ExitKind {

        let input_bytes = input.target_bytes();
        if let Ok(input_str) = str::from_utf8(&input_bytes) {
            let serde_json_parsed: Result<serde_json::Value, serde_json::Error> = serde_json::from_str(input_str);

            let json_parsed = json::parse(input_str);

            if serde_json_parsed.is_ok() != json_parsed.is_ok() {
                eprintln!("mismatch on return type:\n{:?}\n{:?}", serde_json_parsed, json_parsed);
                return ExitKind::Crash;
            }

            // TODO: add state diffing
        }


        ExitKind::Ok
    };

    let mut executor = InProcessExecutor::new(
        &mut harness,
        tuple_list!(edges_observer, time_observer),
        &mut fuzzer,
        &mut state,
        &mut mgr
    ).unwrap();

    let mutator = StdScheduledMutator::new(havoc_mutations());
    let mut stages = tuple_list!(StdMutationalStage::new(mutator));

    if state.corpus().count() < 1 {
        state
            .load_initial_inputs_forced(
                &mut fuzzer,
                &mut executor,
                &mut mgr,
                &["./corpus".into()],
            )
            .unwrap();
    }

    println!("Starting fuzzing loop...");
    fuzzer.fuzz_loop(&mut stages, &mut executor, &mut state, &mut mgr).unwrap();

    println!("Fuzzing finished.");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parsing() {
        let unknown_json_data = r#"
        {
            "name": "Example Widget",
            "version": 1.2,
            "enabled": true,
            "components": [
                { "type": "sensor", "id": "S1", "readings": [ 10, 12, 15 ] },
                { "type": "actuator", "id": "A3" }
            ],
            "metadata": {
                "timestamp": "2025-04-01T09:39:00Z",
                "source": null
            }
        }
    "#;

    // Deserialize into serde_json::Value
    let serde_json_parsed: Result<serde_json::Value, serde_json::Error> = serde_json::from_str(unknown_json_data);

    match serde_json_parsed {
        Ok(data) => {
            println!("Successfully deserialized into Value:\n{:#?}", data);
        }
        Err(e) => {
            panic!("Failed to deserialize JSON: {}", e);
        }
    }

    let json_parsed = json::parse(unknown_json_data);

    match json_parsed {
        Ok(data) => {
            println!("Successfully parsed with 'json' crate:");
            println!("{}", data.pretty(2));
        }
        Err(e) => {
            panic!("Failed to parse JSON with 'json' crate: {}", e);
        }
    }
    }
}
