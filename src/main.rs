use serde_json;
use json;

fn main() {
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
