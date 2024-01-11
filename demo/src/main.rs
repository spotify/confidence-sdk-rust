use confidence::{APIConfig, ConfidenceProvider, ConfidenceResolver, Region};
use open_feature::{EvaluationContext, OpenFeature};

#[tokio::main]
#[warn(unused_must_use)]
async fn main() {
    let api_config = APIConfig {
        api_key: "CLIENT_SECRET".to_string(),
        region: Region::Global,
    };

    let provider = ConfidenceProvider::builder()
        .api_config(api_config)
        .resolver(Box::new(ConfidenceResolver::default()))
        .build();

    let context = EvaluationContext::builder()
        .targeting_key("TARGETING_KEY")
        .build();

    let mut api = OpenFeature::singleton_mut().await;

    api.set_provider(provider).await;

    // wrong type, should return error
    let details_string = api
        .create_client()
        .get_bool_details("FLAG.string_key", Some(&context), None)
        .await;

    // correct type, should return value
    let details_boolean = api
        .create_client()
        .get_bool_details("FLAG.struct_key", Some(&context), None)
        .await;

    println!("details string -> {:?}", details_string);
    println!("details boolean -> {:?}", details_boolean.unwrap().value);
}
