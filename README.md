# Rust Confidence SDK

This repo contains the [Confidence](https://confidence.spotify.com/) Rust SDK and the Confidence OpenFeature provider. We recommend using the OpenFeature Rust SDK to access Confidence feature flags. Use it to consume feature flags from Confidence.

To learn more about the basic concepts (flags, targeting key, evaluation contexts), the [OpenFeature reference documentation](https://openfeature.dev/docs/reference/intro/) can be a useful resource.


## Usage

### Instantiating the ConfidenceProvider and connecting it to OpenFeature

You setup the Confidence Provider once in your app and connect it to OpenFeature .

```rust
// first, we need to setup the api config which contains a `api_key` and a `region`
let api_config = APIConfig { api_key: "YOUR_API_KEY".to_string(), region: YOUR_REGION };

// we can then create a confidence provider using a confidence instance.
let confidence = Confidence::new(api_config)
let provider = ConfidenceProvider::new(confidence);

let mut api = OpenFeature::singleton_mut().await;

// the provider can be set on the open feature api
api.set_provider(provider).await;
```

### Resolving feature flags

After this initial setup we can start accessing the flags using an OpenFeature `client`.
Every time any of flags are asked, the sdk fetches them from the network and resolve the asked property.

```rust
let client = api.create_client();

let context = EvaluationContext::default()
    .with_custom_field("visitor_id", visitor_id)
    .with_custom_field("user_id", user_id);
let details_string = client.get_string_details("my-flag.property", Some(&context), None).await;

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
```

The schema of the property plays a crucial role in resolving the property, if the schema type matches the asked type, the value will be returned otherwise
we expect an `MismatchType` error from the `EvaluationError`.

```rust
// wrong type, should return error
let details_string = confidence.get_flag("hawkflag.message", "default".to_string()).await;
println!("details string -> {:?}", details_string);
```
