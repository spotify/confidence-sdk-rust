use std::collections::HashMap;
use std::sync::Arc;

use crate::{APIConfig, Confidence, ConfidenceValue};

pub trait Contextual {
    fn put_context(&mut self, key: &str, value: ConfidenceValue);
    fn get_context(&self) -> HashMap<String, ConfidenceValue>;
    fn with_context(&self, context: HashMap<String, ConfidenceValue>) -> Confidence;
}

impl Contextual for Confidence {
    fn put_context(&mut self, key: &str, value: ConfidenceValue) {
        self.context.insert(key.to_string(), value);
    }

    fn get_context(&self) -> HashMap<String, ConfidenceValue> {
        let mut context = HashMap::new();
        for (key, value) in self.context.iter() {
            context.insert(key.to_string(), value.clone());
        }

        context
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
            .context(new_context)
            .resolver(Arc::clone(&self.resolver))
            .build()
    }
}

// write tests for the Contextual trait
#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;

    use crate::{APIConfig, Confidence, ConfidenceValue, Region};
    use crate::contextual_confidence::Contextual;

    #[test]
    fn test_put_context() {
        let mut confidence = Confidence::builder()
            .api_config(APIConfig { api_key: "".to_string(), region: Region::EU })
            .resolver(Arc::new(crate::ConfidenceResolver::default()))
            .build();
        confidence.put_context("key", ConfidenceValue::Int(1));
        assert_eq!(confidence.context.get("key"), Some(&ConfidenceValue::Int(1)));
    }

    #[test]
    fn test_with_context() {
        let mut confidence = Confidence::builder()
            .api_config(APIConfig { api_key: "X".to_string(), region: Region::EU })
            .resolver(Arc::new(crate::ConfidenceResolver::default()))
            .build();

        let mut context = HashMap::new();
        context.insert("key".to_string(), ConfidenceValue::Int(1));
        let new_confidence = confidence.with_context(context);
        assert_eq!(new_confidence.context.get("key"), Some(&ConfidenceValue::Int(1)));
    }
}