# Rust Confidence SDK

Rust implementation of the [Confidence](https://confidence.spotify.com/).

## Usage

### Instantiating the Confidence

first, we need to setup the api config which contains a `api_key` and a `region`:

```rust
let api_config = APIConfig { api_key: "YOUR_API_KEY".to_string(), region: YOUR_REGION };

let confidence = Confidence::new(api_config)
```

after this initial setup we can start accessing the flags.
Every time any of flags are asked, the sdk fetches them from the network and resolve the asked property.

The schema of the property plays a crucial role in resolving the property, if the schema type matches the asked type, the value will be returned otherwise
we expect an `MismatchType` error from the `EvaluationError`.

```rust
// wrong type, should return error
let details_string = confidence.get_flag("hawkflag.message", "default".to_string()).await;
println!("details string -> {:?}", details_string);
```
### Send custom Events
we can send custom events to the confidence sdk like following:

```rust
confidence.track("[EVENT-NAME]", HashMap::new());
```