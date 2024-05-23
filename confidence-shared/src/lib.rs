use std::collections::HashMap;
use std::sync::Arc;

use confidence::{APIConfig, Confidence as CoreConfidence, ConfidenceValue};
use confidence::contextual_confidence::Contextual;

pub struct Confidence {
    core_confidence: CoreConfidence
}

impl Confidence {
    pub fn new() -> Self {
        let core_confidence = CoreConfidence::new(
            APIConfig{
                api_key: "API_KEY".to_string(),
                region: confidence::Region::Global
            },
        );
        Confidence {
            core_confidence
        }
    }

    async fn get_flag_string(self: Arc<Confidence>, flag_key: String, default_value: String) -> String {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.spawn(async move {
            resolve_flag(&self.core_confidence, flag_key, default_value).await
        }).await.unwrap()
    }
}

async fn resolve_flag(confidence: &CoreConfidence, flag_key: String, default_value: String) -> String {
    let mut hash_map = HashMap::new();
    hash_map.insert("targeting_key".to_string(), ConfidenceValue::String("Sample".to_string()));
    let confidence = confidence.with_context(hash_map);

    confidence
        .get_flag(&flag_key, default_value)
        .await
        .unwrap()
        .value
}

uniffi::include_scaffolding!("shared");