use std::collections::HashMap;

use spotify_confidence_sdk::{APIConfig, Confidence, ConfidenceValue, Region};
use spotify_confidence_sdk::contextual_confidence::Contextual;
use spotify_confidence_sdk::event_sender::EventSender;

#[tokio::main]
#[warn(unused_must_use)]
async fn main() {
let api_config = APIConfig {
        api_key: "API_KEY".to_string(),
        region: Region::Global,
    };
    let mut context = HashMap::new();
    context.insert("visitor_id".to_string(), ConfidenceValue::String("dennis".to_string()));

    let confidence = Confidence::new(api_config).with_context(context);

    // wrong type, should return error
    let details_string = confidence.get_flag("hawkflag.message", "default".to_string()).await;

    println!("details string -> {:?}", details_string.unwrap().value);

    // send 10 track events and wait 10 seconds
    for i in 0..10 {
        confidence.track("navigate", HashMap::new());
    }
}
