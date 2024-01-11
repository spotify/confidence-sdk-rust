mod flag_schema_deserializer;
mod models;
mod resolve;

pub use crate::models::Region;
pub use crate::models::APIConfig;
use crate::models::ResolvedFlag;
use typed_builder::TypedBuilder;
use crate::models::ResolvedFlags;
use crate::resolve::NetworkFlagResolver;
pub use crate::resolve::ConfidenceResolver;
use async_trait::async_trait;
use models::ResolveError;
use open_feature::{
    self, provider::FeatureProvider, provider::ProviderMetadata, provider::ResolutionDetails,
    EvaluationContext, EvaluationError, StructValue,
};

#[derive(TypedBuilder)]
pub struct ConfidenceProvider {
    #[builder(setter(into))]
    api_config: APIConfig,
    resolver: Box<dyn NetworkFlagResolver + Sync + Send>
}

impl<'a> ConfidenceProvider  {
    async fn fetch_resolved_flags(
        &self,
        _flag_key: &str,
        evaluation_context: &EvaluationContext,
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
        evaluation_context: &EvaluationContext,
    ) -> Result<ResolutionDetails<open_feature::Value>, EvaluationError> {
        let resolved_flags = self
            .fetch_resolved_flags(_flag_key, evaluation_context)
            .await
            .unwrap()
            .flags;

        let flag_segments: Vec<&str> = _flag_key.split(".").collect();
        let flag_name = format!("flags/{}", flag_segments.first().unwrap());
        let property_path = flag_segments[1..].to_vec();

        if resolved_flags.len() == 0 {
            Err(EvaluationError::builder()
                .message("Could not fetch the flag")
                .code(open_feature::EvaluationErrorCode::FlagNotFound)
                .build())
        } else {
            if resolved_flags[0].flag == flag_name {
                // todo - if property path is empty
                self.process_flag(&resolved_flags[0], property_path)
            } else {
                Err(EvaluationError::builder()
                    .message("The fetched flag name doesn't match")
                    .code(open_feature::EvaluationErrorCode::FlagNotFound)
                    .build())
            }
        }
    }

    fn process_flag(
        &self,
        resolved_flag: &ResolvedFlag,
        property_path: Vec<&str>,
    ) -> Result<ResolutionDetails<open_feature::Value>, EvaluationError> {
        let mut last_struct: &open_feature::StructValue = &resolved_flag.value;

        for path in property_path {
            if let Some(value) = last_struct.fields.get(path) {
                if let open_feature::Value::Struct(struct_value) = value {
                    last_struct = struct_value;
                    continue;
                } else {
                    return Ok(ResolutionDetails::builder()
                        .reason(open_feature::EvaluationReason::TargetingMatch)
                        .variant(resolved_flag.variant.clone())
                        .value(value.clone())
                        .build());
                }
            } else {
                return Err(EvaluationError::builder()
                    .message("The fetched flag name doesn't match")
                    .code(open_feature::EvaluationErrorCode::FlagNotFound)
                    .build());
            }
        }

        let value = open_feature::Value::Struct(last_struct.clone());

        return Ok(ResolutionDetails::builder()
            .reason(open_feature::EvaluationReason::Default)
            .variant(resolved_flag.variant.clone())
            .value(value)
            .build());
    }
}

#[async_trait]
#[warn(deprecated)]
#[allow(unused_variables)]
impl FeatureProvider for ConfidenceProvider {
    async fn resolve_int_value(
        &self,
        _flag_key: &str,
        evaluation_context: &EvaluationContext,
    ) -> Result<ResolutionDetails<i64>, EvaluationError> {
        let value = self
            .resolve_value(_flag_key, evaluation_context)
            .await
            .unwrap();

        if let Some(int_value) = value.value.as_i64() {
            Ok(ResolutionDetails::builder()
            .reason(value.reason.unwrap())
            .variant(value.variant.unwrap())
            .value(int_value)
            .build())
        } else {
            let err = EvaluationError {
                code: open_feature::EvaluationErrorCode::TypeMismatch,
                message: Some(format!("schema type is different for {_flag_key}").to_string())
            };
            Err(err)
        }
    }

    async fn resolve_string_value(
        &self,
        _flag_key: &str,
        evaluation_context: &EvaluationContext,
    ) -> Result<ResolutionDetails<String>, EvaluationError> {
        let value = self
            .resolve_value(_flag_key, evaluation_context)
            .await
            .unwrap();

        if let Some(string_value) = value.value.as_str() {
            Ok(ResolutionDetails::builder()
            .reason(value.reason.unwrap())
            .variant(value.variant.unwrap())
            .value(string_value.to_string())
            .build())
        } else {
            let err = EvaluationError {
                code: open_feature::EvaluationErrorCode::TypeMismatch,
                message: Some(format!("schema type is different for {_flag_key}").to_string())
            };
            Err(err)
        }
    }

    async fn resolve_float_value(
        &self,
        _flag_key: &str,
        evaluation_context: &EvaluationContext,
    ) -> Result<ResolutionDetails<f64>, EvaluationError> {
        let value = self
            .resolve_value(_flag_key, evaluation_context)
            .await
            .unwrap();

        if let Some(float_value) = value.value.as_f64() {
            Ok(ResolutionDetails::builder()
            .reason(value.reason.unwrap())
            .variant(value.variant.unwrap())
            .value(float_value)
            .build())
        } else {
            let err = EvaluationError {
                code: open_feature::EvaluationErrorCode::TypeMismatch,
                message: Some(format!("schema type is different for {_flag_key}").to_string())
            };
            Err(err)
        }
    }

    async fn resolve_bool_value(
        &self,
        _flag_key: &str,
        evaluation_context: &EvaluationContext,
    ) -> Result<ResolutionDetails<bool>, EvaluationError> {
        let value = self
            .resolve_value(_flag_key, evaluation_context)
            .await
            .unwrap();

        if let Some(bool_value) = value.value.as_bool() {
            Ok(ResolutionDetails::builder()
            .reason(value.reason.unwrap())
            .variant(value.variant.unwrap())
            .value(value.value.as_bool().unwrap_or_default())
            .build())
        } else {
            let err = EvaluationError {
                code: open_feature::EvaluationErrorCode::TypeMismatch,
                message: Some(format!("schema type is different for {_flag_key}").to_string())
            };
            Err(err)
        }
    }

    async fn resolve_struct_value(
        &self,
        _flag_key: &str,
        evaluation_context: &EvaluationContext,
    ) -> Result<ResolutionDetails<StructValue>, EvaluationError> {
        let value = self
            .resolve_value(_flag_key, evaluation_context)
            .await
            .unwrap();

        if let Some(struct_value) = value.value.as_struct() {
            Ok(ResolutionDetails::builder()
            .reason(value.reason.unwrap())
            .variant(value.variant.unwrap())
            .value(struct_value.clone())
            .build())
        } else {
            let err = EvaluationError {
                code: open_feature::EvaluationErrorCode::TypeMismatch,
                message: Some(format!("schema type is different for {_flag_key}").to_string())
            };
            Err(err)
        }
    }

    fn metadata(&self) -> &ProviderMetadata {
        todo!()
    }
}
