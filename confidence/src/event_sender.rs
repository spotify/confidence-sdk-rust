use std::collections::HashMap;
use std::hash::Hash;

use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value;
use typed_builder::TypedBuilder;

use crate::{Confidence, ConfidenceValue, get_sdk_version, SDK_ID};
use crate::contextual_confidence::Contextual;
use crate::conversion_trait::ToSerdeValueConverter;
use crate::models::SDK;

#[async_trait::async_trait]
pub trait EventSender {
    fn track(&self, name: &str, message: HashMap<String, ConfidenceValue>);
    async fn async_track(&self, name: &str, message: HashMap<String, ConfidenceValue>);
}

#[async_trait::async_trait]
impl EventSender for Confidence {
    fn track(&self, name: &str, message: HashMap<String, ConfidenceValue>) {
        let context = self.context.clone();
        let api_config = self.api_config.clone();
        let event_name = name.to_string().clone();
        tokio::spawn(async move {
            send_event(
                api_config.api_key,
                event_name,
                merge_context(context, message)
            ).await;
        });
    }

    async fn async_track(&self, name: &str, message: HashMap<String, ConfidenceValue>) {
        send_event(
            self.api_config.api_key.clone(),
            name.to_string(),
            merge_context(self.context.clone(), message)
        ).await;
    }
}


fn merge_context(context: HashMap<String, ConfidenceValue>, message: HashMap<String, ConfidenceValue>) -> HashMap<String, Value> {
    let mut context_map: HashMap<String, Value> = context.iter()
        .map(|(key, value)| (key.clone(), value.clone().convert()))
        .collect();
    let message_map: HashMap<String, Value> = message
        .iter()
        .map(|(key, value)| (key.clone(), value.clone().convert()))
        .collect();
    for (key, value) in message_map {
        context_map.insert(key, value);
    }
    context_map
}

async fn send_event(client_secret: String, _name: String, _message: HashMap<String, Value>) {
    let now: DateTime<Utc> = Utc::now();
    let sdk = SDK::builder().id(SDK_ID).version(get_sdk_version()).build();

    let events = Event::builder()
        .event_definition(format!("eventDefinitions/{}", _name))
        .event_time(now)
        .payload(_message)
        .build();

    let req = &EventRequest::builder()
        .client_secret(client_secret)
        .send_time(now)
        .events(Vec::from([events]))
        .sdk(sdk)
        .build();

    let body =
        serde_json::to_string(req).unwrap();
    let client = reqwest::Client::new();
    let response = client
        .post("https://events.confidence.dev/v1/events:publish")
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .body(body.clone())
        .send()
        .await;

    match response {
        Ok(success) => {
            match success.text().await {
                Ok(_) => {

                }
                Err(err) => println!("ERROR ->> {}", err)
            }
        }
        Err(error) => {
            println!("ERROR response ->> {:?}", error)
        }
    }
}

#[derive(Debug, Serialize, TypedBuilder)]
pub struct EventRequest {
    #[builder(setter(into))]
    #[serde(rename = "clientSecret")]
    client_secret: String,
    #[builder(setter(into))]
    events: Vec<Event>,
    #[builder(setter(into))]
    #[serde(rename = "sendTime")]
    send_time: DateTime<Utc>,
    #[builder(setter(into))]
    sdk: SDK
}

#[derive(Debug, Serialize, TypedBuilder)]
pub struct Event {
    #[builder(setter(into))]
    #[serde(rename = "eventDefinition")]
    event_definition: String,
    #[builder(setter(into))]
    #[serde(rename = "eventTime")]
    event_time: DateTime<Utc>,
    #[builder(setter(into))]
    payload: HashMap<String, Value>,
}