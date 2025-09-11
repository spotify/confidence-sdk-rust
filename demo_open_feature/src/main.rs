use open_feature::{EvaluationContext, OpenFeature};
use spotify_confidence_sdk::{APIConfig, Confidence, Region};
use spotify_confidence_openfeature_provider::ConfidenceProvider;

#[tokio::main]
#[warn(unused_must_use)]
async fn main() {
let api_config = APIConfig {
        api_key: "API_KEY".to_string(),
        region: Region::Global,
    };
    let confidence = Confidence::new(api_config);
    let provider = ConfidenceProvider::new(confidence);

    let mut api = OpenFeature::singleton_mut().await;

    api.set_provider(provider).await;

    let client = api.create_client();

    let random_number = rand::random::<u64>();
    let visitor_id = random_number.to_string();

    let context = EvaluationContext::default()
        .with_custom_field("visitor_id", visitor_id)
        .with_custom_field("user_id", "1111111111");
    let details_string = client.get_string_details("hawkflag.message", Some(&context), None).await;
    
    match details_string {
        Ok(details) => {
            println!("Successfully retrieved flag value: {:?}", details.value);
            println!("Flag details: {:?}", details);
        }
        Err(error) => {
            println!("Error retrieving flag: {:?}", error);
            println!("This is expected if the flag 'hawkflag.message' doesn't exist in your Confidence project");
        }
    }
}
