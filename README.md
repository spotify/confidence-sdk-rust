# OpenFeature Rust Confidence Provider
Rust implementation of the [Confidence](https://confidence.spotify.com/) feature provider, to be used in conjunction with the [OpenFeature SDK](https://github.com/open-feature/rust-sdk).

## Usage

### Enabling the provider, setting the evaluation context and resolving flags

`setProvider` makes the openfeature api to be able to use confidence as the provider for flags.

first, we need to setup the api config which contains a `client_secret` and a a `region`, then we can create a provider like following:

```rust
let api_config = APIConfig { api_key: "YOUR_CLIENT_SECRET".to_string(), region: YOUR_REGION };

let provider = ConfidenceProvider::builder()
.api_config(api_config)
.resolver(Box::new(ConfidenceResolver::default()))
.build();
```

at this point we can set the provider for the `OpenFeatureAPI` as following:

```rust
let mut api = OpenFeature::singleton_mut().await;

api.set_provider(provider).await;
```

after this initial setup we can start accessing the flags.
Every time any of flags are asked, the provider fetches them from the network and resolve the asked property.

The schema of the property plays a crucial role in resolving the property, if the schema type matches the asked type, the value will be returned otherwise
we expect an `MismatchType` error from the `EvaluationError`.

```rust
// wrong type, should return error
let details_string = api
.create_client()
.get_bool_details("test.struct-key.string-key", Some(&context), None)
.await;

// correct type, should return value
let details_boolean = api
.create_client()
.get_bool_details("my-test.struct-key.boolean-key", Some(&context), None)
.await;

println!("details string -> {:?}", details_string);
println!("details boolean -> {:?}", details_boolean.unwrap().value);
```

The result of the above code would be:

```shell
details string -> Err(EvaluationError { code: TypeMismatch, message: Some("schema type is different for my-test.struct-key.string-key") })
details boolean -> true
```

### Reading the whole Flag as Struct

It’s also possible to read the whole flag and get the whole flag as a struct like following


```rust
let details_flag = api
.create_client()
.get_struct_details::<MyStructValue>("my-test.struct-key", Some(&context), None)
    .await
    .unwrap();

    println!("details boolean struct -> {:?}", details_flag.value.my_boolean);
    println!("details string struct -> {:?}", details_flag.value.my_string);
}

struct MyStructValue {
    my_boolean: bool,
    my_string: String
}

impl FromStructValue for MyStructValue {
    fn from_struct_value(value: &StructValue) -> anyhow::Result<Self> {
        return Ok(MyStructValue {
            my_boolean: value.fields.get("boolean-key").unwrap().as_bool().unwrap_or_default(),
            my_string: value.fields.get("string-key").unwrap().as_str().unwrap_or_default().to_string(),
        })
    }
}
```
