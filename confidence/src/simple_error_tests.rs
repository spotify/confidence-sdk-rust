#[cfg(test)]
mod tests {
    use crate::{
        APIConfig, Region,
        details::EvaluationReason,
        evaluation_error::EvaluationErrorCode,
        confidence_value::ConfidenceValue,
    };

    #[test]
    fn test_invalid_flag_key_format() {
        // Test that empty flag key is handled gracefully
        let flag_segments: Vec<&str> = "".split(".").collect();
        assert!(flag_segments.first().is_none() || flag_segments.first() == Some(&""));
    }

    #[test]
    fn test_flag_key_parsing() {
        // Test normal flag key parsing
        let flag_segments: Vec<&str> = "flag.property".split(".").collect();
        assert_eq!(flag_segments.first(), Some(&"flag"));
        assert_eq!(flag_segments.len(), 2);
        
        // Test single segment flag key
        let flag_segments: Vec<&str> = "flag".split(".").collect();
        assert_eq!(flag_segments.first(), Some(&"flag"));
        assert_eq!(flag_segments.len(), 1);
    }

    #[test]
    fn test_evaluation_reason_fallback() {
        // Test unwrap_or behavior simulation
        let maybe_reason: Option<EvaluationReason> = None;
        let final_reason = maybe_reason.unwrap_or(EvaluationReason::Static);
        assert_eq!(final_reason, EvaluationReason::Static);
        
        let some_reason: Option<EvaluationReason> = Some(EvaluationReason::TargetingMatch);
        let final_reason = some_reason.unwrap_or(EvaluationReason::Static);
        assert_eq!(final_reason, EvaluationReason::TargetingMatch);
    }

    #[test]
    fn test_variant_fallback() {
        // Test variant fallback behavior
        let maybe_variant: Option<String> = None;
        let final_variant = maybe_variant.unwrap_or("unknown".to_string());
        assert_eq!(final_variant, "unknown");
        
        let some_variant: Option<String> = Some("test_variant".to_string());
        let final_variant = some_variant.unwrap_or("unknown".to_string());
        assert_eq!(final_variant, "test_variant");
    }

    #[test]
    fn test_confidence_value_type_checking() {
        // Test that ConfidenceValue type checking works as expected
        let string_value = ConfidenceValue::String("hello".to_string());
        let int_value = ConfidenceValue::Int(42);
        let bool_value = ConfidenceValue::Bool(true);
        
        // Test type conversion attempts
        assert!(string_value.as_str().is_some());
        assert!(string_value.as_i64().is_none()); // Should fail type conversion
        
        assert!(int_value.as_i64().is_some());
        assert!(int_value.as_str().is_none()); // Should fail type conversion
        
        assert!(bool_value.as_bool().is_some());
        assert!(bool_value.as_str().is_none()); // Should fail type conversion
    }

    #[test]
    fn test_evaluation_error_creation() {
        // Test that we can create evaluation errors properly
        use crate::evaluation_error::EvaluationError;
        
        let error = EvaluationError {
            code: EvaluationErrorCode::TypeMismatch,
            message: Some("test error message".to_string()),
        };
        
        assert_eq!(error.code, EvaluationErrorCode::TypeMismatch);
        assert_eq!(error.message, Some("test error message".to_string()));
        
        let error_with_formatted_message = EvaluationError {
            code: EvaluationErrorCode::FlagNotFound,
            message: Some(format!("schema type is different for {}", "test.flag")),
        };
        
        assert!(error_with_formatted_message.message.unwrap().contains("test.flag"));
    }

    #[test] 
    fn test_api_config_creation() {
        // Test that API config can be created properly
        let config = APIConfig {
            api_key: "test_key".to_string(),
            region: Region::Global,
        };
        
        assert_eq!(config.api_key, "test_key");
        assert_eq!(config.region, Region::Global);
    }
}
