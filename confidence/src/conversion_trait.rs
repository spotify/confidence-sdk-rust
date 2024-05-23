use serde_json::Value;

use crate::confidence_value::{ConfidenceValue, StructValue};

pub trait TypeConversionTrait {
  fn process(&self, value: &ConfidenceValue, default: &Self) -> Self;
}

impl TypeConversionTrait for i64 {
  fn process(&self, value: &ConfidenceValue, default: &i64) -> i64 {
    return value.as_i64().unwrap_or(default.clone());
  }
}

impl TypeConversionTrait for bool {
  fn process(&self, value: &ConfidenceValue, default: &bool) -> bool {
    return value.as_bool().unwrap_or(default.clone());
  }
}

impl TypeConversionTrait for f64 {
  fn process(&self, value: &ConfidenceValue, default: &f64) -> f64 {
    return value.as_f64().unwrap_or(default.clone());
  }
}

impl TypeConversionTrait for StructValue {
  fn process(&self, value: &ConfidenceValue, default: &StructValue) -> StructValue {
    return value.as_struct().unwrap_or(default).clone()
  }
}


impl TypeConversionTrait for String {
  fn process(&self, value: &ConfidenceValue, default: &String) -> String {
    return value.as_str().unwrap_or(default).to_string();
  }
}

pub trait ToSerdeValueConverter {
  fn convert(self) -> Value;
}

impl ToSerdeValueConverter for ConfidenceValue {
  fn convert(self) -> Value {
    match self {
      ConfidenceValue::Bool(value) => {
        Value::from(value)
      }
      ConfidenceValue::Int(value) => {
        Value::from(value)
      }
      ConfidenceValue::Float(value) => {
        Value::from(value)
      }
      ConfidenceValue::String(value) => {
        Value::from(value)
      }
      ConfidenceValue::Array(value) => {
        Value::Array(value.iter().map(|item| item.clone().convert()).collect())
      }
      ConfidenceValue::Struct(value) => {
        let map = value
            .fields
            .iter()
            .map(|(key, value)| (key.clone(), value.clone().convert()))
            .collect();
        Value::Object(map)
      }
    }
  }
}