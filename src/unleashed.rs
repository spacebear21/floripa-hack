use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION};
use reqwest::{Client, Error as ReqwestError};
use serde::{Deserialize, Serialize};
use std::error::Error;

pub fn init_unleashed_client(unleashed_api_key: &str) -> Result<Client, Box<dyn Error>> {
    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, HeaderValue::from_str("application/json")?);
    let mut auth_value = HeaderValue::from_str(&format!("Bearer {}", unleashed_api_key))?;
    auth_value.set_sensitive(true);
    headers.insert(AUTHORIZATION, auth_value);
    Ok(reqwest::ClientBuilder::new()
        .default_headers(headers)
        .build()?)
}

pub async fn get_unleashed_balance(client: &Client) -> Result<UnleashedBalance, ReqwestError> {
    let res = client
        .get("https://unleashed.chat/api/v1/account/balance")
        .send()
        .await?;
    res.json().await
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UnleashedBalance {
    balance: f64,
    balance_currency: String,
}
