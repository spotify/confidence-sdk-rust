use serde::{Deserialize, Serialize};
use serde_json::Value;
use typed_builder::TypedBuilder;
use std::collections::HashMap;

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
pub enum Region {
    US,
    EU,
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
    pub value: open_feature::StructValue,
    pub reason: String,
}

trait FlagValueConversion<T> {
    fn into_value(self, schema: &Option<FlagSchema>) -> T;
}

impl FlagValueConversion<open_feature::StructValue> for Option<Value> {
    fn into_value(self, schema: &Option<FlagSchema>) -> open_feature::StructValue {
        if let Some(schema) = schema {
            let schema = &schema.schema;
            match self {
                Some(value) => {
                    if let Value::Object(value_map) = value {
                        let new_map: HashMap<String, open_feature::Value> = value_map
                            .into_iter()
                            .map(|(key, value)| {
                                let converted_value = match schema[&key].clone() {
                                    SchemaType::BoolType => open_feature::Value::Bool(
                                        value.as_bool().unwrap_or_default(),
                                    ),
                                    SchemaType::IntType => {
                                        open_feature::Value::Int(value.as_i64().unwrap_or_default())
                                    }
                                    SchemaType::DoubleType => open_feature::Value::Float(
                                        value.as_f64().unwrap_or_default(),
                                    ),
                                    SchemaType::StringType => open_feature::Value::String(
                                        value.as_str().unwrap_or_default().to_string(),
                                    ),
                                    SchemaType::StructType(struct_value) => {
                                        open_feature::Value::Struct(Some(value).into_value(
                                            &Some(FlagSchema {
                                                schema: *struct_value,
                                            }),
                                        ))
                                    }
                                };
                                (key, converted_value)
                            })
                            .collect();
                        open_feature::StructValue { fields: new_map }
                    } else {
                        open_feature::StructValue::default()
                    }
                }
                None => open_feature::StructValue::default(),
            }
        } else {
            open_feature::StructValue::default()
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
            Region::GLOBAL => "https://resolver.confidence.dev".to_string(),
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
