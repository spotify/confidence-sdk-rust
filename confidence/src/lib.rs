use std::collections::HashMap;
use std::sync::Arc;

use typed_builder::TypedBuilder;

use conversion_trait::TypeConversionTrait;
use details::EvaluationDetails;
use evaluation_error::EvaluationError;

pub use crate::confidence_value::ConfidenceValue;
use crate::confidence_value::StructValue;
use crate::details::EvaluationReason;
use crate::evaluation_error::EvaluationErrorCode;
pub use crate::models::APIConfig;
pub use crate::models::Region;
use crate::models::ResolvedFlag;
use crate::models::ResolvedFlags;
use crate::models::ResolveError;
pub use crate::resolve::ConfidenceResolver;
use crate::resolve::NetworkFlagResolver;

mod flag_schema_deserializer;
pub mod models;
pub mod resolve;
pub mod confidence_value;
pub mod evaluation_error;
pub mod details;
mod conversion_trait;
pub mod contextual_confidence;
pub mod event_sender;

pub static SDK_ID: &str = "SDK_ID_RUST_CONFIDENCE";

pub fn get_sdk_version() -> String {
    let version = "0.1.0";
    return version.to_string();
}

#[derive(TypedBuilder)]
pub struct Confidence {
    #[builder(setter(into))]
    pub api_config: APIConfig,
    #[builder(default, setter(into))]
    context: HashMap<String, ConfidenceValue>,
    resolver: Arc<dyn NetworkFlagResolver + Sync + Send>
}

impl Confidence {
    pub fn new(api_config: APIConfig) -> Self {
        let mut map = HashMap::new();
        map.insert("targeting_key".to_string(), ConfidenceValue::String("Sample".to_string()));
        Self {
            api_config,
            context: map,
            resolver: Arc::new(ConfidenceResolver::default())
        }
    }

    async fn fetch_resolved_flags(
        &self,
        _flag_key: &str,
        evaluation_context: &HashMap<String, ConfidenceValue>,
    ) -> Result<ResolvedFlags, ResolveError> {
        self.resolver
            .resolve(
                &self.api_config,
                [_flag_key.to_string()].into(),
                evaluation_context,
            )
            .await
    }
    async fn resolve_value(
        &self,
        _flag_key: &str,
        evaluation_context: &HashMap<String, ConfidenceValue>,
    ) -> Result<EvaluationDetails<ConfidenceValue>, EvaluationError> {
        let resolved_flags_result = self
            .fetch_resolved_flags(_flag_key, evaluation_context)
            .await;

        let resolved_flags = match resolved_flags_result {
            Ok(result) => result.flags,
            Err(e) => {
                return Err(EvaluationError::builder()
                    .message(&format!("Failed to fetch resolved flags: {:?}", e))
                    .code(EvaluationErrorCode::FlagNotFound)
                    .build());
            }
        };

        let flag_segments: Vec<&str> = _flag_key.split(".").collect();
        let flag_name = match flag_segments.first() {
            Some(name) => format!("flags/{}", name),
            None => {
                return Err(EvaluationError::builder()
                    .message("Invalid flag key format")
                    .code(EvaluationErrorCode::FlagNotFound)
                    .build());
            }
        };
        let property_path = flag_segments[1..].to_vec();

        if resolved_flags.len() == 0 {
            Err(EvaluationError::builder()
                .message("Could not fetch the flag")
                .code(EvaluationErrorCode::FlagNotFound)
                .build())
        } else {
            if resolved_flags[0].flag == flag_name {
                // todo - if property path is empty
                self.process_flag(&resolved_flags[0], property_path)
            } else {
                Err(EvaluationError::builder()
                    .message("The fetched flag name doesn't match")
                    .code(EvaluationErrorCode::FlagNotFound)
                    .build())
            }
        }
    }

    fn process_flag(
        &self,
        resolved_flag: &ResolvedFlag,
        property_path: Vec<&str>,
    ) -> Result<EvaluationDetails<ConfidenceValue>, EvaluationError> {
        let mut last_struct: &StructValue = &resolved_flag.value;

        for path in property_path {
            if let Some(value) = last_struct.fields.get(path) {
                if let ConfidenceValue::Struct(struct_value) = value {
                    last_struct = struct_value;
                    continue;
                } else {
                    return Ok(EvaluationDetails::builder()
                        .reason(EvaluationReason::TargetingMatch)
                        .variant(resolved_flag.variant.clone())
                        .value(value.clone())
                        .build());
                }
            } else {
                return Err(EvaluationError::builder()
                    .message("The fetched flag name doesn't match")
                    .code(EvaluationErrorCode::FlagNotFound)
                    .build());
            }
        }

        let value = ConfidenceValue::Struct(last_struct.clone());

        return Ok(EvaluationDetails::builder()
            .reason(EvaluationReason::Default)
            .variant(resolved_flag.variant.clone())
            .value(value)
            .build());
    }

    pub async fn get_flag<T: TypeConversionTrait>(
        &self,
        _flag_key: &str,
        default_value: T) -> Result<EvaluationDetails<T>, EvaluationError> {
        let value = match self
            .resolve_value(_flag_key, &self.context)
            .await {
            Ok(val) => val,
            Err(e) => return Err(e),
        };

        if let Some(int_value) = value.value.as_type(&default_value) {
            Ok(EvaluationDetails::builder()
            .reason(value.reason.unwrap_or(EvaluationReason::Default))
            .variant(value.variant.unwrap_or("unknown".to_string()))
            .value(int_value)
            .build())
        } else {
            let err = EvaluationError {
                code: EvaluationErrorCode::TypeMismatch,
                message: Some(format!("schema type is different for {_flag_key}").to_string())
            };
            Err(err)
        }
    }

}

#[cfg(test)]
mod simple_error_tests;
