pub mod data;
pub mod meters;
pub mod topics;

#[cfg(test)]
mod tests;

use governor::{
    Quota, RateLimiter,
    clock::QuantaClock,
    state::{InMemoryState, NotKeyed},
};
use reqwest::Client;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::{num::NonZero, sync::Arc};
use thiserror::Error;
use tracing::{debug, trace};

pub use {data::data, meters::meters, topics::topics};

const BASE_URL: &str = "https://api.dentcloud.io/v1";
const RATELIMIT_PER_SECOND: NonZero<u32> = NonZero::new(5).unwrap();
const RATELIMIT_BURST: NonZero<u32> = NonZero::new(5).unwrap();

const API_HEADER: &str = "x-api-key";
const KEY_HEADER: &str = "x-key-id";

/// Internal type use to serialize errors to return to the end user.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApiError {
    pub success: bool,
    pub error: String,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Received success field equaled to false.")]
    Api(String),
    #[error("Failed network request to meter.")]
    Http(#[from] reqwest::Error),
    #[error("Failed to serialize.")]
    Serialization(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub struct Session {
    api_key: String,
    client: Client,
    key_id: String,
    rate_limiter: Arc<RateLimiter<NotKeyed, InMemoryState, QuantaClock>>,
}

impl Session {
    pub fn new(api_key: String, key_id: String) -> Self {
        let quota = Quota::per_second(RATELIMIT_PER_SECOND).allow_burst(RATELIMIT_BURST);
        Self {
            api_key,
            client: Client::new(),
            key_id,
            rate_limiter: Arc::new(RateLimiter::direct(quota)),
        }
    }

    async fn send<T, U>(&self, query: &[T]) -> Result<U>
    where
        T: Serialize,
        U: DeserializeOwned,
    {
        let request = self
            .client
            .get(BASE_URL)
            .header(API_HEADER, &self.api_key)
            .header(KEY_HEADER, &self.key_id)
            .query(query);

        self.rate_limiter.until_ready().await;
        debug!("Sending request to DentCloud.");
        let response = request.send().await?.error_for_status()?;
        let text = &response.text().await?;
        dbg!("text {}", text);

        trace!("Decoding response text.");
        match serde_json::from_str::<U>(text) {
            Ok(object) => Ok(object),
            Err(error) => {
                // Try to deserialize into error struct instead.
                match serde_json::from_str::<ApiError>(text) {
                    Ok(api_error) => Err(Error::Api(api_error.error)),
                    // We don't care about the inner error of the 2nd failed deserialization, just
                    // the first one.
                    Err(_error_serialization_error) => Err(Error::Serialization(error)),
                }
            }
        }
    }
}
