#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use open_feature::{EvaluationContext, FromStructValue, OpenFeature, StructValue};

    struct MyStructValue {
        fields: HashMap<String, open_feature::Value>,
    }

    impl FromStructValue for MyStructValue {
        fn from_struct_value(value: &StructValue) -> anyhow::Result<Self> {
            return Ok(MyStructValue {
                fields: value.fields.clone(),
            });
        }
    }

    use crate::{
        models::{NetworkResolvedFlags, ResolveError, ResolvedFlags},
        resolve::MockNetworkFlagResolver,
        APIConfig, ConfidenceProvider,
    };

    async fn setup_provider() -> open_feature::Client {
        let config = APIConfig {
            api_key: "".to_string(),
            region: crate::Region::Global,
        };
        let mut mock_resolver = MockNetworkFlagResolver::new();

        mock_resolver.expect_resolve().returning(|_, _, _| {
            Box::pin(async move { resolve_response("test-flag".to_string()) })
        });
        let provider = ConfidenceProvider::builder()
            .api_config(config)
            .resolver(Arc::new(mock_resolver))
            .build();

        let mut api = OpenFeature::singleton_mut().await;

        api.set_provider(provider).await;
        return api.create_client();
    }

    #[tokio::test]
    async fn resovle_boolean_value() {
        let context = EvaluationContext::builder()
            .targeting_key("3poj234lknwfklnasflkaflakjlkejelkfjlkj")
            .build();

        let client = setup_provider().await;

        let value = client
            .get_bool_value("test-flag.boolean-key", Some(&context), None)
            .await
            .unwrap();
        assert_eq!(value, true);
    }

    #[tokio::test]
    async fn resovle_nested_boolean_value() {
        let context = EvaluationContext::builder()
            .targeting_key("3poj234lknwfklnasflkaflakjlkejelkfjlkj")
            .build();

        let client = setup_provider().await;

        let value = client
            .get_bool_value("test-flag.struct-key.boolean-key", Some(&context), None)
            .await
            .unwrap();

        assert_eq!(value, true);
    }

    #[tokio::test]
    async fn resolve_whole_object() {
        let context = EvaluationContext::builder()
            .targeting_key("3poj234lknwfklnasflkaflakjlkejelkfjlkj")
            .build();

        let client = setup_provider().await;

        let value = client
            .get_struct_value::<MyStructValue>("test-flag", Some(&context), None)
            .await
            .unwrap();

        assert_eq!(value.fields.len(), 5);
    }

    #[tokio::test]
    async fn resolve_whole_object_ints() {
        let context = EvaluationContext::builder()
            .targeting_key("3poj234lknwfklnasflkaflakjlkejelkfjlkj")
            .build();

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
        let context = EvaluationContext::builder()
            .targeting_key("3poj234lknwfklnasflkaflakjlkejelkfjlkj")
            .build();

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

    // Error handling tests for the new robust error handling
    #[tokio::test]
    async fn test_resolve_value_network_error() {
        let config = APIConfig {
            api_key: "test_key".to_string(),
            region: crate::Region::Global,
        };
        let mut mock_resolver = MockNetworkFlagResolver::new();

        // Mock a network error
        mock_resolver.expect_resolve().returning(|_, _, _| {
            Box::pin(async move { 
                Err(ResolveError::NetworkError)
            })
        });

        let confidence = crate::Confidence::builder()
            .api_config(config)
            .resolver(Arc::new(mock_resolver))
            .build();

        let result = confidence.resolve_value("test.flag", &HashMap::new()).await;
        
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.code, crate::evaluation_error::EvaluationErrorCode::FlagNotFound);
        assert!(error.message.is_some());
    }

    #[tokio::test]
    async fn test_resolve_value_serialization_error() {
        let config = APIConfig {
            api_key: "test_key".to_string(),
            region: crate::Region::Global,
        };
        let mut mock_resolver = MockNetworkFlagResolver::new();

        // Mock a serialization error
        mock_resolver.expect_resolve().returning(|_, _, _| {
            Box::pin(async move { 
                Err(ResolveError::SerializationError)
            })
        });

        let confidence = crate::Confidence::builder()
            .api_config(config)
            .resolver(Arc::new(mock_resolver))
            .build();

        let result = confidence.resolve_value("test.flag", &HashMap::new()).await;
        
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.code, crate::evaluation_error::EvaluationErrorCode::FlagNotFound);
    }

    #[tokio::test]
    async fn test_resolve_value_invalid_flag_key() {
        let config = APIConfig {
            api_key: "test_key".to_string(),
            region: crate::Region::Global,
        };
        let mock_resolver = MockNetworkFlagResolver::new();

        let confidence = crate::Confidence::builder()
            .api_config(config)
            .resolver(Arc::new(mock_resolver))
            .build();

        // Test with empty flag key (should cause split to return empty vec)
        let result = confidence.resolve_value("", &HashMap::new()).await;
        
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.code, crate::evaluation_error::EvaluationErrorCode::FlagNotFound);
        assert!(error.message.unwrap().contains("Invalid flag key format"));
    }

    #[tokio::test]
    async fn test_get_flag_type_mismatch() {
        let config = APIConfig {
            api_key: "test_key".to_string(),
            region: crate::Region::Global,
        };
        let mut mock_resolver = MockNetworkFlagResolver::new();

        // Mock returning a string value
        mock_resolver.expect_resolve().returning(|_, _, _| {
            Box::pin(async move { 
                let mut resolved_flags = ResolvedFlags::default();
                resolved_flags.flags.push(ResolvedFlag {
                    flag: "flags/test".to_string(),
                    variant: Some("variant1".to_string()),
                    value: crate::confidence_value::ConfidenceValue::String("hello".to_string()),
                    reason: Some(crate::details::EvaluationReason::TargetingMatch),
                });
                Ok(resolved_flags)
            })
        });

        let confidence = crate::Confidence::builder()
            .api_config(config)
            .resolver(Arc::new(mock_resolver))
            .build();

        // Try to get string value as integer (should cause type mismatch)
        let result: Result<crate::details::EvaluationDetails<i64>, _> = confidence.get_flag("test.flag", 42i64).await;
        
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.code, crate::evaluation_error::EvaluationErrorCode::TypeMismatch);
        assert!(error.message.unwrap().contains("schema type is different for test.flag"));
    }

    #[tokio::test]
    async fn test_get_flag_propagates_resolve_error() {
        let config = APIConfig {
            api_key: "test_key".to_string(),
            region: crate::Region::Global,
        };
        let mut mock_resolver = MockNetworkFlagResolver::new();

        // Mock a network error that should propagate through get_flag
        mock_resolver.expect_resolve().returning(|_, _, _| {
            Box::pin(async move { 
                Err(ResolveError::NetworkError)
            })
        });

        let confidence = crate::Confidence::builder()
            .api_config(config)
            .resolver(Arc::new(mock_resolver))
            .build();

        let result: Result<crate::details::EvaluationDetails<String>, _> = confidence.get_flag("test.flag", "default".to_string()).await;
        
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.code, crate::evaluation_error::EvaluationErrorCode::FlagNotFound);
    }

    #[tokio::test]
    async fn test_get_flag_success_with_fallback_values() {
        let config = APIConfig {
            api_key: "test_key".to_string(),
            region: crate::Region::Global,
        };
        let mut mock_resolver = MockNetworkFlagResolver::new();

        // Mock returning a flag with missing optional fields
        mock_resolver.expect_resolve().returning(|_, _, _| {
            Box::pin(async move { 
                let mut resolved_flags = ResolvedFlags::default();
                resolved_flags.flags.push(ResolvedFlag {
                    flag: "flags/test".to_string(),
                    variant: None, // Missing variant - should use fallback
                    value: crate::confidence_value::ConfidenceValue::String("hello".to_string()),
                    reason: None, // Missing reason - should use fallback
                });
                Ok(resolved_flags)
            })
        });

        let confidence = crate::Confidence::builder()
            .api_config(config)
            .resolver(Arc::new(mock_resolver))
            .build();

        let result: Result<crate::details::EvaluationDetails<String>, _> = confidence.get_flag("test.flag", "default".to_string()).await;
        
        assert!(result.is_ok());
        let details = result.unwrap();
        assert_eq!(details.value, "hello");
        assert_eq!(details.reason, Some(crate::details::EvaluationReason::Static)); // Fallback reason
        assert_eq!(details.variant, Some("unknown".to_string())); // Fallback variant
    }
}
