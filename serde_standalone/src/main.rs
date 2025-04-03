#![allow(static_mut_refs)]

use std::path::PathBuf;

use libafl::{
    corpus::{CachedOnDiskCorpus, Corpus, OnDiskCorpus},
    events::SimpleEventManager,
    executors::{inprocess::InProcessExecutor, ExitKind},
    feedback_or, feedback_or_fast,
    feedbacks::{AflMapFeedback, CrashFeedback, TimeFeedback, TimeoutFeedback},
    fuzzer::{Fuzzer, StdFuzzer},
    inputs::{BytesInput, HasTargetBytes},
    monitors::{MultiMonitor, TuiMonitor, SimpleMonitor},
    mutators::{havoc_mutations, StdScheduledMutator},
    observers::{HitcountsMapObserver, StdMapObserver, TimeObserver},
    prelude::CanTrack,
    schedulers::QueueScheduler,
    stages::mutational::StdMutationalStage,
    state::{HasCorpus, StdState},
};
use libafl_bolts::tuples::tuple_list;
use serde_json;

use libafl_targets::coverage::EDGES_MAP;
use libafl_targets::sancov_pcguard;
fn main() {
    let mon = SimpleMonitor::new(|s| println!("{s}"));
    // let mon = TuiMonitor::builder()
    // .title("Serde Standalone")
    // .enhanced_graphics(true)
    // .build();
    let mut mgr = SimpleEventManager::new(mon);
    let edges_observer =
        HitcountsMapObserver::new(unsafe { StdMapObserver::new("edges", &mut EDGES_MAP) });

    let time_observer = TimeObserver::new("time");

    let map_feedback = AflMapFeedback::new(&edges_observer);
    let mut feedback = feedback_or!(map_feedback, TimeFeedback::new(&time_observer));

    let mut objective = feedback_or_fast!(CrashFeedback::new(), TimeoutFeedback::new());

    let mut state = StdState::new(
        libafl_bolts::rands::StdRand::with_seed(0),
        CachedOnDiskCorpus::<BytesInput>::new("/tmp/corpus", 1024).unwrap(),
        OnDiskCorpus::new(PathBuf::from("/tmp/crashes")).unwrap(),
        &mut feedback,
        &mut objective,
    )
    .unwrap();

    let scheduler = QueueScheduler::new();

    let mut fuzzer = StdFuzzer::new(scheduler, feedback, objective);

    let mut harness = |input: &BytesInput| -> ExitKind {
        let input_bytes = input.target_bytes();
        if let Ok(input_str) = str::from_utf8(&input_bytes) {
            let serde_json_parsed: Result<serde_json::Value, serde_json::Error> =
                serde_json::from_str(input_str);

            if let Ok(parsed_value) = serde_json_parsed {
                // Serialize the parsed value back to a string
                if let Ok(serde_string) = serde_json::to_string(&parsed_value) {
                    // Deserialize the serialized string back to a value
                    let round_trip: Result<serde_json::Value, serde_json::Error> =
                        serde_json::from_str(&serde_string);

                    // Check if the round-trip value matches the original parsed value
                    if let Ok(round_trip_value) = round_trip {
                        if round_trip_value != parsed_value {
                            // eprintln!("Failed roundtrip: {}\n{}\n",round_trip_value, parsed_value);
                            return ExitKind::Crash;
                        }
                    }
                }
            }
        }
        ExitKind::Ok
    };

    let mut executor = InProcessExecutor::new(
        &mut harness,
        tuple_list!(edges_observer, time_observer),
        &mut fuzzer,
        &mut state,
        &mut mgr,
    )
    .unwrap();

    let mutator = StdScheduledMutator::new(havoc_mutations());
    let mut stages = tuple_list!(StdMutationalStage::new(mutator));

    if state.corpus().count() < 1 {
        state
            .load_initial_inputs_forced(&mut fuzzer, &mut executor, &mut mgr, &["./corpus".into()])
            .unwrap();
    }

    println!("Starting fuzzing loop...");
    fuzzer
        .fuzz_loop(&mut stages, &mut executor, &mut state, &mut mgr)
        .unwrap();

    println!("Fuzzing finished.");
}
