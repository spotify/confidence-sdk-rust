use std::collections::HashMap;

use crate::models::{FlagSchema, SchemaType};
use serde::{
    de::Visitor,
    de::{IntoDeserializer, MapAccess},
    Deserialize,
};
use serde_json::Value;

impl<'de> Deserialize<'de> for FlagSchema {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(FlagSchemaVisitor)
    }
}

struct FlagSchemaVisitor;

impl<'de> Visitor<'de> for FlagSchemaVisitor {
    type Value = FlagSchema;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Deserializing the schemas")
    }

    fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
    where
        V: MapAccess<'de>,
    {
        let mut schema: HashMap<String, SchemaType> = HashMap::new();

        if let Some((_, entry_value)) = map.next_entry::<String, Value>()? {
            // "schema: {...}"
            if let Value::Object(entry_map) = entry_value {
                let entry_map: HashMap<String, Value> = entry_map.into_iter().collect();

                for (key, value) in &entry_map {
                    if let Some(_) = value.get("boolSchema") {
                        schema.insert(key.into(), SchemaType::BoolType);
                        continue;
                    }

                    if let Some(_) = value.get("intSchema") {
                        schema.insert(key.into(), SchemaType::IntType);
                        continue;
                    }

                    if let Some(_) = value.get("stringSchema") {
                        schema.insert(key.into(), SchemaType::StringType);
                        continue;
                    }

                    if let Some(_) = value.get("doubleSchema") {
                        schema.insert(key.into(), SchemaType::DoubleType);
                        continue;
                    }

                    if let Some(Value::Object(struct_schema)) = value.get("structSchema") {
                        let struct_schema: HashMap<String, Value> =
                            struct_schema.clone().into_iter().collect();
                        let struct_type =
                            FlagSchema::deserialize(struct_schema.into_deserializer())
                                .map_err(serde::de::Error::custom)?;
                        schema.insert(
                            key.into(),
                            SchemaType::StructType(Box::new(struct_type.schema)),
                        );
                        continue;
                    }
                }
            }
        }

        Ok(FlagSchema { schema })
    }
}
