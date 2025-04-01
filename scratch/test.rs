use serde_json::{json, Value};
use std::collections::BTreeMap;
use json::JsonValue;

#[macro_use]
extern crate json;

fn sort_json(value: &JsonValue) -> JsonValue {
    match value {
        JsonValue::Object(obj) => {
            // Convert to BTreeMap for sorting
            let mut sorted: BTreeMap<String, JsonValue> = BTreeMap::new();

            // Add each key-value pair, recursively sorting nested objects
            for (key, value) in obj.iter() {
                sorted.insert(key.to_string(), sort_json(value));
            }

            // Convert back to JsonValue::Object
            let mut result = json::object::Object::new();
            for (key, value) in sorted {
                result.insert(&key, value);
            }

            JsonValue::Object(result)
        },
        JsonValue::Array(arr) => {
            // Process arrays by recursively sorting their elements
            let mut result = json::array![];
            for item in arr.iter() {
                result.push(sort_json(item)).unwrap();
            }
            result
        },
        // For other types, just clone the value
        _ => value.clone(),
    }
}

fn sort_json_value(value: Value) -> Value {
    match value {
        Value::Object(map) => {
            // Sort this level by converting to BTreeMap
            let sorted_map: BTreeMap<_, _> = map.into_iter()
                .map(|(k, v)| (k, sort_json_value(v)))  // Recursively sort nested values
                .collect();

            // Convert back to a Value::Object
            Value::Object(sorted_map.into_iter().collect())
        },
        Value::Array(arr) => {
            // Recursively sort values in arrays
            Value::Array(arr.into_iter().map(sort_json_value).collect())
        },
        // Other JSON types (strings, numbers, booleans, null) don't need sorting
        other => other,
    }
}

fn main() {
    let data = json!({
        "z": "last",
        "a": "first",
        "nested": {
            "y": 2,
            "x": 1,
            "deep": {
                "c": true,
                "b": null,
                "a": [3, 2, {"z": 0, "a": 1}]
            }
        },
        "m": "middle"
    });
    let data2:JsonValue = json::parse(r#"
    {
        "z": "last",
        "a": "first",
        "nested": {
            "y": 2,
            "x": 1,
            "deep": {
                "c": true,
                "b": null,
                "a": [3, 2, {"z": 0, "a": 1}]
            }
        },
        "m": "middle"
    }
        "#).unwrap();

    let sorted = sort_json_value(data);
    let sorted_json = serde_json::to_string_pretty(&sorted).unwrap();
    println!("{}", sorted_json);


    let sorted2 = sort_json(&data2);
    let sorted_json2 = json::stringify_pretty(sorted2, 2);
    println!("{}", sorted_json2);
    if sorted_json == sorted_json2 {
        println!("we matched");
    }
}