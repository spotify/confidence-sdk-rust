use std::collections::HashMap;
use open_feature::{EvaluationContext, OpenFeature};

use confidence::{APIConfig, Confidence, Region};
use provider::ConfidenceProvider;

#[tokio::main]
#[warn(unused_must_use)]
async fn main() {
let api_config = APIConfig {
        api_key: "API_KEY".to_string(),
        region: Region::Global,
    };
    let confidence = Confidence::new(api_config);
    let provider = ConfidenceProvider::new(confidence);

    let context = EvaluationContext {
        targeting_key: Some("TARGETING_KEY".to_string()),
        custom_fields: HashMap::new()
    };

    let mut api = OpenFeature::singleton_mut().await;

    api.set_provider(provider).await;

    // wrong type, should return error
    let details_string = api
        .create_client().get_string_details("hawkflag.message", Some(&context), None).await;

    println!("details string -> {:?}", details_string.unwrap().value);
}
