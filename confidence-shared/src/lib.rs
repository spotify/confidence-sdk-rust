use std::collections::HashMap;
use std::sync::Arc;

use lazy_static::lazy_static;
use tokio::sync::Mutex;

use confidence::{APIConfig as CoreAPIConfig, Confidence as CoreConfidence, ConfidenceValue as CoreConfidenceValue, Region as CoreRegion};
use confidence::confidence_value::StructValue;
use confidence::contextual_confidence::Contextual;
use confidence::event_sender::EventSender;

pub struct Confidence {
    core_confidence: Arc<Mutex<CoreConfidence>>
}
#[derive(Clone)]
pub enum ConfidenceValue {
    Bool{value: bool},
    Int{value: i64},
    String{value: String},
    Float{value: f64},
    List{value: Vec<ConfidenceValue>},
    Struct{value: HashMap<String, ConfidenceValue>},
}

#[derive(Clone)]
pub struct APIConfig {
    api_key: String,
    region: Region,
}

lazy_static! {
    static ref RUNTIME: tokio::runtime::Runtime = tokio::runtime::Runtime::new().unwrap();
}

#[derive(Clone)]
pub enum Region {
    US,
    EU,
    Global,
}

impl Confidence {
    pub fn new(api_config: APIConfig) -> Self {
        let core_confidence =
            Arc::new(Mutex::new(CoreConfidence::new(api_config.into())));
        Confidence { core_confidence }
    }

    pub fn put_context(&self, key: String, value: ConfidenceValue) {
        let mut context = self.core_confidence.blocking_lock();
        context.put_context(&key, value.convert());
    }

    async fn get_flag_string(&self, flag_key: String, default_value: String) -> String {
        let confidence = Arc::clone(&self.core_confidence);
        RUNTIME.spawn(async move {
            let confidence = confidence.lock().await;
            confidence
                .get_flag(&flag_key, default_value)
                .await
                .unwrap()
                .value
        }).await.unwrap()
    }

    fn with_context(&self, context: HashMap<String, ConfidenceValue>) -> Arc<Confidence> {
        let core_confidence = self.core_confidence
            .blocking_lock()
            .with_context(context.iter().map(|(key, value)| (key.clone(), value.clone().convert()))
                .collect());
        let confidence = Confidence {
            core_confidence: Arc::new(Mutex::new(core_confidence))
        };
        for (key, value) in context {
            confidence.put_context(key, value);
        }
        Arc::new(confidence)
    }

    fn track(&self, event_name: String, message: HashMap<String, ConfidenceValue>) {
        let confidence = Arc::clone(&self.core_confidence);
        RUNTIME.block_on(async move {
            let confidence = confidence.lock().await;
            confidence.track(event_name.as_str(), HashMap::new());
        });
    }
}

trait ConvertToCoreValue {
    fn convert(self) -> CoreConfidenceValue;
}

impl ConvertToCoreValue for ConfidenceValue {
    fn convert(self) -> CoreConfidenceValue {
        match self {
            ConfidenceValue::Bool { value } => {
                CoreConfidenceValue::Bool(value)
            }
            ConfidenceValue::Int { value} => {
                CoreConfidenceValue::Int(value)
            }
            ConfidenceValue::String { value} => {
                CoreConfidenceValue::String(value)
            }
            ConfidenceValue::Float { value} => {
                CoreConfidenceValue::Float(value)
            }
            ConfidenceValue::List { value} => {
                CoreConfidenceValue::Array(value.into_iter().map(|v| v.convert()).collect())
            }
            ConfidenceValue::Struct { value } => {
                let map = value;
                let fields = map.iter().map(|(key, value)| (key.clone(), value.clone().convert())).collect();
                CoreConfidenceValue::Struct(StructValue { fields })
            }
        }
    }
}

impl Into<CoreAPIConfig> for APIConfig {
    fn into(self) -> CoreAPIConfig {
        CoreAPIConfig {
            api_key: self.api_key,
            region: self.region.into()
        }
    }
}

impl Into<CoreRegion> for Region {
    fn into(self) -> CoreRegion {
        match self {
            Region::US => CoreRegion::US,
            Region::EU => CoreRegion::EU,
            Region::Global => CoreRegion::Global,
        }
    }
}

uniffi::include_scaffolding!("shared");