use std::collections::HashMap;

use async_trait::async_trait;
use mockall::automock;
use serde_json::Value;
use crate::confidence_value::ConfidenceValue;
use crate::models::APIConfig;
use crate::models::NetworkResolvedFlags;
use crate::models::ResolveError;
use crate::models::ResolveRequest;
use crate::models::ResolvedFlags;
use crate::models::APIURL;
use crate::models::SDK;
use crate::{get_sdk_version, SDK_ID};
use crate::conversion_trait::ToSerdeValueConverter;

#[derive(Clone, Default)]
pub struct ConfidenceResolver;

impl ConfidenceResolver {

    async fn make_request(
        &self,
        config: &APIConfig,
        flags: Vec<String>,
        _evaluation_context: &HashMap<String, ConfidenceValue>,
    ) -> Result<NetworkResolvedFlags, ResolveError> {
        let flags: Vec<String> = flags.into_iter().filter_map(|flag| {
            let flag_name: Vec<&str> = flag.split(".").collect();
            flag_name.first().map(|name| format!("flags/{}", name))
        }).collect();

        let context: HashMap<String, Value> = _evaluation_context
            .iter()
            .map(|(key, value)| (key.clone(), value.clone().convert()))
            .collect();

        let sdk = SDK::builder().id(SDK_ID).version(get_sdk_version()).build();

        let resolve_request = &ResolveRequest::builder()
        .client_secret(config.api_key.clone())
        .evaluation_context(context)
        .apply(true)
        .sdk(sdk)
        .flags(flags)
        .build();

        let body = match serde_json::to_string(resolve_request) {
            Ok(json) => json,
            Err(_) => return Err(ResolveError::SerializationError),
        };

        let client = reqwest::Client::new();
        let response = client
            .post(format!("{}/v1/flags:resolve", config.region.url()))
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .body(body)
            .send()
            .await?;

        match response.text().await {
            Ok(body) => {
                let resolved_flags: serde_json::Result<NetworkResolvedFlags> =
                    serde_json::from_str(&body);
                match resolved_flags {
                    Ok(resolved) => Result::Ok(resolved),
                    Err(err) => {
                        println!("ERROR ->> {}", err);
                        Err(ResolveError::SerializationError)
                    }
                }
            }
            Err(err) => Err(ResolveError::NetworkError(err)),
        }
    }
}

#[async_trait]
#[automock]
pub trait NetworkFlagResolver {
    async fn resolve(
        &self,
        config: &APIConfig,
        flags: Vec<String>,
        evaluation_context: &HashMap<String, ConfidenceValue>,
    ) -> Result<ResolvedFlags, ResolveError>;
}

#[async_trait]
impl NetworkFlagResolver for ConfidenceResolver {
    async fn resolve(
        &self,
        config: &APIConfig,
        flags: Vec<String>,
        evaluation_context: &HashMap<String, ConfidenceValue>,
    ) -> Result<ResolvedFlags, ResolveError> {
        let network_response = self.make_request(config, flags, evaluation_context).await?;
        Ok(network_response.into())
    }
}