use async_trait::async_trait;
use open_feature::{self, EvaluationContext, EvaluationError, provider::FeatureProvider, provider::ProviderMetadata, provider::ResolutionDetails, StructValue};
use typed_builder::TypedBuilder;

use confidence::{Confidence, contextual_confidence::Contextual};
pub use confidence::confidence_value::StructValue as ConfidenceStructValue;

use crate::conversion_traits::{ConvertContext, ResolutionDetailsConverter};
use crate::conversion_traits::ToValueConverter;

pub mod conversion_traits;

#[derive(TypedBuilder)]
pub struct ConfidenceProvider {
    metadata: ProviderMetadata,
    confidence: Confidence
}

impl ConfidenceProvider {
    pub fn new(confidence: Confidence) -> Self {
        Self {
            metadata: ProviderMetadata::builder().name("ConfidenceProvider").build(),
            confidence
        }
    }
}

#[async_trait]
#[warn(deprecated)]
#[allow(unused_variables)]
impl FeatureProvider for ConfidenceProvider {
    fn metadata(&self) -> &ProviderMetadata {
        &self.metadata
    }

    async fn resolve_bool_value(
        &self,
        _flag_key: &str,
        evaluation_context: &EvaluationContext,
    ) -> Result<ResolutionDetails<bool>, EvaluationError> {
        let context = evaluation_context.convert();
        let new_confidence = self.confidence.with_context(context);
        let result = new_confidence.get_flag(_flag_key, bool::default()).await;
        return result.convert()
    }

    async fn resolve_int_value(
        &self,
        _flag_key: &str,
        evaluation_context: &EvaluationContext,
    ) -> Result<ResolutionDetails<i64>, EvaluationError> {
        let context = evaluation_context.convert();
        let new_confidence = self.confidence.with_context(context);
        let result = new_confidence.get_flag(_flag_key, i64::default()).await;
        return result.convert()

    }

    async fn resolve_float_value(
        &self,
        _flag_key: &str,
        evaluation_context: &EvaluationContext,
    ) -> Result<ResolutionDetails<f64>, EvaluationError> {
        let context = evaluation_context.convert();
        let new_confidence = self.confidence.with_context(context);
        let result = new_confidence.get_flag(_flag_key, f64::default()).await;
        return result.convert()

    }

    async fn resolve_string_value(
        &self,
        _flag_key: &str,
        evaluation_context: &EvaluationContext,
    ) -> Result<ResolutionDetails<String>, EvaluationError> {
        let context = evaluation_context.convert();
        let new_confidence = self.confidence.with_context(context);
        let result = new_confidence.get_flag(_flag_key, "".to_string()).await;
        return result.convert()
    }

    async fn resolve_struct_value(
        &self,
        _flag_key: &str,
        evaluation_context: &EvaluationContext,
    ) -> Result<ResolutionDetails<StructValue>, EvaluationError> {
        let context = evaluation_context.convert();
        let new_confidence = self.confidence.with_context(context);
        let result = new_confidence.get_flag(_flag_key, ConfidenceStructValue::default()).await;
        let returned_type = result.convert();
        return match returned_type {
            Ok(details) => {
                Ok(ResolutionDetails {
                    value: details.value.convert(),
                    variant: details.variant,
                    reason: details.reason,
                    flag_metadata: details.flag_metadata,
                })
            }
            Err(error) => {
                Err(error)
            }
        }
    }
}