use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::{APIConfig, Confidence, ConfidenceResolver, ConfidenceValue};

pub trait Contextual {
    fn put_context(&mut self, key: &str, value: ConfidenceValue);
    fn get_context(&self) -> Arc<RwLock<HashMap<String, ConfidenceValue>>>;
    fn with_context(&self, context: HashMap<String, ConfidenceValue>) -> Confidence;
}

impl Contextual for Confidence {
    fn put_context(&mut self, key: &str, value: ConfidenceValue) {
        self.context.insert(key.to_string(), value);
    }

    fn get_context(&self) -> Arc<RwLock<HashMap<String, ConfidenceValue>>> {
        let mut context = HashMap::new();
        for (key, value) in self.context.iter() {
            context.insert(key.to_string(), value.clone());
        }

        Arc::new(RwLock::new(context))
    }

    fn with_context(&self, context: HashMap<String, ConfidenceValue>) -> Confidence {
        let api_config = (&self.api_config).clone();
        let new_context: HashMap<String, ConfidenceValue> = context
            .iter()
            .map(|(key, value)| (key.to_string(), value.clone()))
            .collect();

        Confidence::builder()
            .api_config(APIConfig {
                api_key: api_config.api_key,
                region: api_config.region
            })
            .resolver(Box::new(ConfidenceResolver::default()))
            .context(new_context)
            .build()
    }
}

// write tests for the Contextual trait
#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{APIConfig, Confidence, ConfidenceValue, Contextual, Region};

    #[test]
    fn test_put_context() {
        let mut confidence = Confidence::builder()
            .api_config(APIConfig { api_key: "".to_string(), region: Region::EU })
            .resolver(Box::new(crate::ConfidenceResolver::default()))
            .build();
        confidence.put_context("key", ConfidenceValue::Int(1));
        assert_eq!(confidence.context.get("key"), Some(&ConfidenceValue::Int(1)));
    }

    #[test]
    fn test_with_context() {
        let mut confidence = Confidence::builder()
            .api_config(APIConfig { api_key: "X".to_string(), region: Region::EU })
            .resolver(Box::new(crate::ConfidenceResolver::default()))
            .build();

        let mut context = HashMap::new();
        context.insert("key".to_string(), ConfidenceValue::Int(1));
        let new_confidence = confidence.with_context(context);
        assert_eq!(new_confidence.context.get("key"), Some(&ConfidenceValue::Int(1)));
    }
}