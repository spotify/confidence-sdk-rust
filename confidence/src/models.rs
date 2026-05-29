use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use crate::confidence_value::StructValue;
use crate::confidence_value::ConfidenceValue;
use typed_builder::TypedBuilder;

#[derive(Debug)]
pub enum ResolveError {
    NetworkError(reqwest::Error),
    SerializationError,
    // Add more variants for other custom errors if needed
}

#[derive(Clone)]
pub struct APIConfig {
    pub api_key: String,
    pub region: Region,
}

#[allow(unused_variables)]
#[derive(Clone)]
#[derive(Debug, PartialEq)]
pub enum Region {
    US,
    EU,
    Global,
}

#[allow(unused_variables)]
#[derive(Deserialize, Debug, Clone)]
pub struct NetworkResolvedFlags {
    #[serde(rename = "resolvedFlags")]
    pub resolve_flags: Vec<NetworkResolvedFlag>,
    #[serde(rename = "resolveToken")]
    pub resolve_token: String,
}

#[allow(unused_variables)]
#[derive(Deserialize, Debug, Clone)]
pub struct NetworkResolvedFlag {
    pub flag: String,
    pub variant: String,
    pub value: Option<Value>,
    pub reason: String,
    #[serde(rename = "flagSchema")]
    pub flag_schema: Option<FlagSchema>,
}

#[derive(Debug, Clone)]
pub struct FlagSchema {
    pub schema: HashMap<String, SchemaType>,
}

#[allow(unused_variables)]
#[derive(Debug, Clone, Default)]
pub struct ResolvedFlags {
    pub resolve_token: String,
    pub flags: Vec<ResolvedFlag>,
}

#[allow(unused_variables)]
#[derive(Debug, Clone)]
pub struct ResolvedFlag {
    pub flag: String,
    pub variant: String,
    pub value: StructValue,
    pub reason: String,
}

trait FlagValueConversion<T> {
    fn into_value(self, schema: &Option<FlagSchema>) -> T;
}

impl FlagValueConversion<StructValue> for Option<Value> {
    fn into_value(self, schema: &Option<FlagSchema>) -> StructValue {
        if let Some(schema) = schema {
            let schema = &schema.schema;
            match self {
                Some(value) => {
                    if let Value::Object(value_map) = value {
                        let new_map: HashMap<String, ConfidenceValue> = value_map
                            .into_iter()
                            .filter_map(|(key, value)| {
                                // A null property carries no value: the variant doesn't define one,
                                // so resolution must fall back to the call-site default (the same
                                // behaviour as a missing property, and what every other Confidence
                                // SDK does). Omitting the field here makes the downstream path
                                // lookup miss, which surfaces the default. Without this, the
                                // scalar arms below coerce null to the type's zero value via
                                // `unwrap_or_default()` (false / 0 / 0.0 / ""), silently replacing
                                // the call-site default.
                                if value.is_null() {
                                    return None;
                                }
                                let converted_value = match schema[&key].clone() {
                                    SchemaType::BoolType => ConfidenceValue::Bool(
                                        value.as_bool().unwrap_or_default(),
                                    ),
                                    SchemaType::IntType => {
                                        ConfidenceValue::Int(value.as_i64().unwrap_or_default())
                                    }
                                    SchemaType::DoubleType => ConfidenceValue::Float(
                                        value.as_f64().unwrap_or_default(),
                                    ),
                                    SchemaType::StringType => ConfidenceValue::String(
                                        value.as_str().unwrap_or_default().to_string(),
                                    ),
                                    SchemaType::StructType(struct_value) => {
                                        ConfidenceValue::Struct(Some(value).into_value(&Some(
                                            FlagSchema {
                                                schema: *struct_value,
                                            },
                                        )))
                                    }
                                };
                                Some((key, converted_value))
                            })
                            .collect();
                        StructValue { fields: new_map }
                    } else {
                        StructValue::default()
                    }
                }
                None => StructValue::default(),
            }
        } else {
            StructValue::default()
        }
    }
}

impl Into<ResolvedFlag> for NetworkResolvedFlag {
    fn into(self) -> ResolvedFlag {
        ResolvedFlag {
            flag: self.flag,
            variant: self.variant,
            value: self.value.into_value(&self.flag_schema),
            reason: self.reason,
        }
    }
}

impl Into<ResolvedFlags> for NetworkResolvedFlags {
    fn into(self) -> ResolvedFlags {
        ResolvedFlags {
            resolve_token: self.resolve_token,
            flags: self
                .resolve_flags
                .into_iter()
                .map(|flag| flag.into())
                .collect(),
        }
    }
}

#[allow(unused_variables)]
#[derive(Debug, Clone, Deserialize)]
pub enum SchemaType {
    IntType,
    DoubleType,
    StringType,
    BoolType,
    StructType(Box<HashMap<String, SchemaType>>),
}

pub trait APIURL {
    fn url(&self) -> String;
}

impl APIURL for Region {
    fn url(&self) -> String {
        match self {
            Region::EU => "https://resolver.eu.confidence.dev".to_string(),
            Region::US => "https://resolver.us.confidence.dev".to_string(),
            Region::Global => "https://resolver.confidence.dev".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, TypedBuilder)]
pub struct SDK {
    #[builder(setter(into))]
    id: String,
    #[builder(setter(into))]
    version: String,
}

#[derive(Debug, Serialize, Deserialize, TypedBuilder)]
pub struct ResolveRequest {
    #[builder(setter(into))]
    client_secret: String,
    #[builder(setter(into))]
    apply: bool,
    #[builder(setter(into))]
    sdk: SDK,
    #[builder(setter(into))]
    evaluation_context: HashMap<String, Value>,
    #[builder(setter(into))]
    flags: Vec<String>,
}

impl From<reqwest::Error> for ResolveError {
    fn from(error: reqwest::Error) -> ResolveError {
        ResolveError::NetworkError(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn resolved_flags_with_nulls() -> ResolvedFlags {
        let json = r#"
        {
          "resolvedFlags": [
            {
              "flag": "flags/test-flag",
              "variant": "flags/test-flag/variants/treatment",
              "value": {
                "boolean-key": null,
                "string-key": "served",
                "struct-key": {
                  "nested-null-key": null,
                  "nested-boolean-key": true
                }
              },
              "flagSchema": {
                "schema": {
                  "boolean-key": { "boolSchema": {} },
                  "string-key": { "stringSchema": {} },
                  "struct-key": {
                    "structSchema": {
                      "schema": {
                        "nested-null-key": { "boolSchema": {} },
                        "nested-boolean-key": { "boolSchema": {} }
                      }
                    }
                  }
                }
              },
              "reason": "RESOLVE_REASON_MATCH"
            }
          ],
          "resolveToken": ""
        }
        "#;

        let network: NetworkResolvedFlags = serde_json::from_str(json).unwrap();
        network.into()
    }

    #[test]
    fn null_valued_property_is_omitted_so_resolution_falls_back_to_default() {
        let resolved = resolved_flags_with_nulls();
        let fields = &resolved.flags[0].value.fields;

        // A null property must be absent so the path lookup misses and the call-site
        // default is served — not coerced to the type's zero value (false here).
        assert!(
            !fields.contains_key("boolean-key"),
            "null property should be omitted, found {:?}",
            fields.get("boolean-key")
        );

        // Non-null siblings are untouched.
        assert_eq!(
            fields.get("string-key"),
            Some(&ConfidenceValue::String("served".to_string()))
        );
    }

    #[test]
    fn null_valued_nested_property_is_omitted() {
        let resolved = resolved_flags_with_nulls();
        let nested = resolved.flags[0]
            .value
            .fields
            .get("struct-key")
            .and_then(|v| v.as_struct())
            .expect("struct-key should be present");

        assert!(
            !nested.fields.contains_key("nested-null-key"),
            "null nested property should be omitted, found {:?}",
            nested.fields.get("nested-null-key")
        );
        assert_eq!(
            nested.fields.get("nested-boolean-key"),
            Some(&ConfidenceValue::Bool(true))
        );
    }
}
