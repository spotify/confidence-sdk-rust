#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;

    use serde_json;
    use open_feature::{EvaluationContext, OpenFeature, StructValue};
    use open_feature::provider::ProviderMetadata;
    use spotify_confidence_sdk::{APIConfig, Confidence, Region};

    struct MyStructValue {
        fields: HashMap<String, open_feature::Value>,
    }

    impl From<StructValue> for MyStructValue {
        fn from(value: StructValue) -> Self {
            return MyStructValue {
                fields: value.fields.clone(),
            };
        }
    }

    use spotify_confidence_sdk::{
        models::{NetworkResolvedFlags, ResolveError, ResolvedFlags},
        resolve::MockNetworkFlagResolver
    };
    use crate::ConfidenceProvider;

    async fn setup_provider() -> open_feature::Client {
        let api_config = APIConfig {
            api_key: "".to_string(),
            region: Region::Global,
        };

        let mut mock_resolver = MockNetworkFlagResolver::new();
        mock_resolver.expect_resolve().returning(|_, _, _| {
            Box::pin(async move { resolve_response("test-flag".to_string()) })
        });

        let confidence = Confidence::builder()
            .api_config(api_config)
            .resolver(Arc::new(mock_resolver))
            .build();


        let provider = ConfidenceProvider::builder()
            .confidence(confidence)
            .metadata(ProviderMetadata::default())
            .build();

        let mut api = OpenFeature::singleton_mut().await;

        api.set_provider(provider).await;
        return api.create_client();
    }

    #[tokio::test]
    async fn resovle_boolean_value() {
        let context = EvaluationContext {
            targeting_key: Some("3poj234lknwfklnasflkaflakjlkejelkfjlkj".to_string()),
            custom_fields: HashMap::new()
        };

        let client = setup_provider().await;

        let value = client
            .get_bool_value("test-flag.boolean-key", Some(&context), None)
            .await
            .unwrap();
        assert_eq!(value, true);
    }

    #[tokio::test]
    async fn resovle_nested_boolean_value() {
        let context = EvaluationContext {
            targeting_key: Some("3poj234lknwfklnasflkaflakjlkejelkfjlkj".to_string()),
            custom_fields: HashMap::new()
        };

        let client = setup_provider().await;

        let value = client
            .get_bool_value("test-flag.struct-key.boolean-key", Some(&context), None)
            .await
            .unwrap();

        assert_eq!(value, true);
    }

    #[tokio::test]
    async fn resolve_whole_object() {
        let context = EvaluationContext {
            targeting_key: Some("3poj234lknwfklnasflkaflakjlkejelkfjlkj".to_string()),
            custom_fields: HashMap::new()
        };

        let client = setup_provider().await;

        let value = client
            .get_struct_value::<MyStructValue>("test-flag", Some(&context), None)
            .await
            .unwrap();

        assert_eq!(value.fields.len(), 5);
    }

    #[tokio::test]
    async fn resolve_whole_object_ints() {
        let context = EvaluationContext {
            targeting_key: Some("3poj234lknwfklnasflkaflakjlkejelkfjlkj".to_string()),
            custom_fields: HashMap::new()
        };

        let client = setup_provider().await;

        let value = client
            .get_struct_value::<MyStructValue>("test-flag", Some(&context), None)
            .await
            .unwrap();

        assert_eq!(value.fields["integer-key"], open_feature::Value::Int(40));
        if let open_feature::Value::Struct(struct_value) = value.fields["struct-key"].clone() {
            assert_eq!(
                struct_value.fields["integer-key"],
                open_feature::Value::Int(23)
            );
        } else {
            assert_eq!(1, 0);
        }
    }

    #[tokio::test]
    async fn resovle_double_nested_boolean_value() {
        let context = EvaluationContext {
            targeting_key: Some("3poj234lknwfklnasflkaflakjlkejelkfjlkj".to_string()),
            custom_fields: HashMap::new()
        };

        let client = setup_provider().await;

        let value = client
            .get_bool_value(
                "test-flag.struct-key.nested-struct-key.nested-boolean-key",
                Some(&context),
                None,
            )
            .await
            .unwrap();

        assert_eq!(value, true);
    }

    fn resolve_response(flag: String) -> Result<ResolvedFlags, ResolveError> {
        let json_data = r#"
        {
 "resolvedFlags": [
 {
  "flag": "flags/{flag}",
  "variant": "flags/{flag}/variants/treatment",
  "value": {
   "struct-key": {
    "boolean-key": true,
    "string-key": "treatment-struct",
    "double-key": 123.23,
    "integer-key": 23,
	"nested-struct-key": {
		"nested-boolean-key": true
	}
   },
   "boolean-key": true,
   "string-key": "treatment",
   "double-key": 20.203,
   "integer-key": 40
  },
  "flagSchema": {
   "schema": {
    "struct-key": {
     "structSchema": {
      "schema": {
       "boolean-key": {
        "boolSchema": {}
       },
       "string-key": {
        "stringSchema": {}
       },
       "double-key": {
        "doubleSchema": {}
       },
       "integer-key": {
        "intSchema": {}
       },
	   "nested-struct-key": {
		"structSchema": {
			"schema": {
				"nested-boolean-key": {
					"boolSchema": {}
				}
			}
		}
	   }
      }
     }
    },
    "boolean-key": {
     "boolSchema": {}
    },
    "string-key": {
     "stringSchema": {}
    },
    "double-key": {
     "doubleSchema": {}
    },
    "integer-key": {
     "intSchema": {}
    }
   }
  },
  "reason": "RESOLVE_REASON_MATCH"
 }],
 "resolveToken": ""
}
        "#
            .to_string()
            .replace("{flag}", flag.as_ref());

        let network_flags: NetworkResolvedFlags = serde_json::from_str(&json_data).unwrap();
        return Ok(network_flags.into());
    }
}