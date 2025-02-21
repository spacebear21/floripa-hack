use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION};
use serde::{Deserialize, Serialize};
use std::error::Error;

pub struct UnleashedClient {
    client: reqwest::Client,
}

impl UnleashedClient {
    pub fn new(unleashed_api_key: &str) -> Result<UnleashedClient, Box<dyn Error>> {
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, HeaderValue::from_str("application/json")?);
        let mut auth_value = HeaderValue::from_str(&format!("Bearer {}", unleashed_api_key))?;
        auth_value.set_sensitive(true);
        headers.insert(AUTHORIZATION, auth_value);
        let client = reqwest::ClientBuilder::new()
            .default_headers(headers)
            .build()?;
        Ok(UnleashedClient { client })
    }

    pub async fn get_balance(&self) -> Result<Balance, reqwest::Error> {
        let res = self
            .client
            .get("https://unleashed.chat/api/v1/account/balance")
            .send()
            .await?;
        res.json().await
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Balance {
    balance: f64,
    balance_currency: String,
}
