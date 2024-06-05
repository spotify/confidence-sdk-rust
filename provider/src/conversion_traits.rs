use std::collections::HashMap;

use spotify_confidence_sdk::details::EvaluationDetails as ConfidenceEvaluationDetails;
use spotify_confidence_sdk::evaluation_error::EvaluationError as ConfidenceError;
use open_feature::{self, EvaluationContext, EvaluationContextFieldValue, EvaluationError, EvaluationErrorCode, FlagMetadata, FlagMetadataValue, StructValue, Value};
use open_feature::provider::ResolutionDetails;
use spotify_confidence_sdk::ConfidenceValue;

pub use spotify_confidence_sdk::confidence_value::StructValue as ConfidenceStructValue;
use spotify_confidence_sdk::details::{EvaluationReason as ConfidenceReason, EvaluationReason};
use spotify_confidence_sdk::evaluation_error::EvaluationErrorCode as ConfidenceErrorCode;

pub trait ToValueConverter {
    fn convert(self) -> StructValue;
}

impl ToValueConverter for ConfidenceStructValue {
    fn convert(self) -> StructValue {
        let fields = self.fields.iter().map(|(key, value)| (key.clone(), value.clone().convert())).collect();
        StructValue {
            fields
        }
    }
}

trait ToOFValueConverter {
    fn convert(self) -> open_feature::Value;
}

impl ToOFValueConverter for ConfidenceValue {
    fn convert(self) -> open_feature::Value {
        match self {
            ConfidenceValue::Bool(value) => {
                open_feature::Value::Bool(value)
            }
            ConfidenceValue::Int(value) => {
                open_feature::Value::Int(value)
            }
            ConfidenceValue::Float(value) => {
                open_feature::Value::Float(value)
            }
            ConfidenceValue::String(value) => {
                open_feature::Value::String(value)
            }
            ConfidenceValue::Array(value) => {
                let list = value
                    .iter()
                    .map(|item| item.clone().convert() )
                    .collect();

                open_feature::Value::Array(list)
            }
            ConfidenceValue::Struct(struct_value) => {
                let fields = struct_value
                    .fields
                    .iter()
                    .map(|(key, value)| (key.clone(), value.clone().convert()))
                    .collect();

                open_feature::Value::Struct(
                    StructValue {
                    fields
                })

            }
        }
    }
}

pub trait ToConfidenceValue {
    fn convert(self) -> ConfidenceValue;
}

impl ToConfidenceValue for Value {
    fn convert(self) -> ConfidenceValue {
        match self {
            Value::Bool(value) => {
                ConfidenceValue::Bool(value)
            }
            Value::Int(value) => {
                ConfidenceValue::Int(value)
            }
            Value::Float(value) => {
                ConfidenceValue::Float(value)
            }
            Value::String(value) => {
                ConfidenceValue::String(value)
            }
            Value::Array(value) => {
                let list = value
                    .iter()
                    .map(|item| item.clone().convert())
                    .collect();
                ConfidenceValue::Array(list)
            }
            Value::Struct(struct_value) => {
                let fields = struct_value
                    .fields
                    .iter()
                    .map(|(key, value)| (key.clone(), value.clone().convert()))
                    .collect();
                ConfidenceValue::Struct(ConfidenceStructValue { fields })
            }
        }
    }
}

pub trait EvaluationErrorCodeConverter {
    fn convert(self) -> EvaluationErrorCode;
}

impl EvaluationErrorCodeConverter for ConfidenceErrorCode {
    fn convert(self) -> EvaluationErrorCode {
        match self {
            ConfidenceErrorCode::ProviderNotReady => {
                EvaluationErrorCode::ProviderNotReady
            }
            ConfidenceErrorCode::FlagNotFound => {
                EvaluationErrorCode::FlagNotFound
            }
            ConfidenceErrorCode::ParseError => {
                EvaluationErrorCode::ParseError
            }
            ConfidenceErrorCode::TypeMismatch => {
                EvaluationErrorCode::TypeMismatch
            }
            ConfidenceErrorCode::TargetingKeyMissing => {
                EvaluationErrorCode::TargetingKeyMissing
            }
            ConfidenceErrorCode::InvalidContext => {
                EvaluationErrorCode::InvalidContext
            }
            ConfidenceErrorCode::General(general) => {
                EvaluationErrorCode::General(general)
            }
        }
    }
}

pub trait ReasonConverter {
    fn convert(self) -> Option<open_feature::EvaluationReason>;
}

impl ReasonConverter for Option<ConfidenceReason> {
    fn convert(self) -> Option<open_feature::EvaluationReason> {
        self.and_then(|reason| {
            let of_reason = match reason {
                EvaluationReason::Static => {
                    open_feature::EvaluationReason::Static
                }
                EvaluationReason::Default => {
                    open_feature::EvaluationReason::Default
                }
                EvaluationReason::TargetingMatch => {
                    open_feature::EvaluationReason::TargetingMatch
                }
                EvaluationReason::Split => {
                    open_feature::EvaluationReason::Split
                }
                EvaluationReason::Cached => {
                    open_feature::EvaluationReason::Cached
                }
                EvaluationReason::Disabled => {
                    open_feature::EvaluationReason::Cached
                }
                EvaluationReason::Unknown => {
                    open_feature::EvaluationReason::Unknown
                }
                EvaluationReason::Error => {
                    open_feature::EvaluationReason::Error
                }
                EvaluationReason::Other(other) => {
                    open_feature::EvaluationReason::Other(other)
                }
            };
            Some(of_reason)
        })
    }
}

pub trait FlagMetadataConverter {
    fn convert(self) -> Option<FlagMetadata>;
}

impl FlagMetadataConverter for Option<spotify_confidence_sdk::details::FlagMetadata> {
    fn convert(self) -> Option<FlagMetadata> {
        self.and_then(|metadata| {
            let field_values = metadata.values.iter().map(|(key, field_value)| (key.clone(), field_value.clone().convert())).collect();
            Some(
                FlagMetadata {
                    values: field_values
                }
            )
    })
    }
}

pub trait EvaluationContextValueConverter {
    fn convert(self) -> ConfidenceValue;
}

impl EvaluationContextValueConverter for EvaluationContextFieldValue {
    fn convert(self) -> ConfidenceValue {
        match self {
            EvaluationContextFieldValue::Bool(value) => {
                ConfidenceValue::Bool(value)
            }
            EvaluationContextFieldValue::Int(value) => {
                ConfidenceValue::Int(value)
            }
            EvaluationContextFieldValue::Float(value) => {
                ConfidenceValue::Float(value)
            }
            EvaluationContextFieldValue::String(value) => {
                ConfidenceValue::String(value)
            }
            EvaluationContextFieldValue::DateTime(value) => {
                ConfidenceValue::String(value.to_string())
            }
            EvaluationContextFieldValue::Struct(value) => {
                let of_struct_value =  value.clone().downcast::<StructValue>().unwrap();
                let fields: HashMap<String, ConfidenceValue> = (*of_struct_value)
                    .fields
                    .iter()
                    .map(|(key, value)| (key.clone(), value.clone().convert())).collect();
                let struct_value = ConfidenceStructValue { fields };
                ConfidenceValue::Struct(struct_value)
            }
        }
    }
}

pub trait FlagValueConverter {
    fn convert(self) -> FlagMetadataValue;
}

impl FlagValueConverter for spotify_confidence_sdk::details::FlagMetadataValue {
    fn convert(self) -> FlagMetadataValue {
        match self {
            spotify_confidence_sdk::details::FlagMetadataValue::Bool(value) => {
                FlagMetadataValue::Bool(value)
            }
            spotify_confidence_sdk::details::FlagMetadataValue::Int(value) => {
                FlagMetadataValue::Int(value)
            }
            spotify_confidence_sdk::details::FlagMetadataValue::Float(value) => {
                FlagMetadataValue::Float(value)
            }
            spotify_confidence_sdk::details::FlagMetadataValue::String(value) => {
                FlagMetadataValue::String(value)
            }
        }
    }
}

pub trait ConvertContext {
    fn convert(&self) -> HashMap<String, ConfidenceValue>;
}

impl ConvertContext for &EvaluationContext {
    fn convert(&self) -> HashMap<String, ConfidenceValue> {
        let mut new_map: HashMap<String, ConfidenceValue> = HashMap::new();
        for (key, value) in &self.custom_fields {
            new_map.insert(key.into(), value.clone().convert());
        }
        match &self.targeting_key {
            None => {}
            Some(targeting_key) => {
                new_map.insert("targeting_key".into(), ConfidenceValue::String(targeting_key.to_string()));
            }
        };
        return new_map;
    }
}

pub trait ResolutionDetailsConverter<T> {
    fn convert(self) -> Result<ResolutionDetails<T>, EvaluationError>;
}

impl <T> ResolutionDetailsConverter<T> for Result<ConfidenceEvaluationDetails<T>, ConfidenceError> {
    fn convert(self) -> Result<ResolutionDetails<T>, EvaluationError> {
        return match self {
            Ok(details) => {
                Ok(ResolutionDetails{
                    value: details.value,
                    variant: details.variant,
                    reason: details.reason.convert(),
                    flag_metadata: details.flag_metadata.convert(),
                })
            }
            Err(error) => {
                Err(EvaluationError{
                    code: error.code.convert(),
                    message: error.message,
                })
            }
        }
    }
}