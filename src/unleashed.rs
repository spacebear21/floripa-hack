use futures_util::StreamExt;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::error::Error;

pub struct UnleashedClient {
    client: reqwest::Client,
}

impl UnleashedClient {
    pub fn new(unleashed_api_key: &str) -> Result<UnleashedClient, Box<dyn Error + Send + Sync>> {
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

    pub async fn ask_llm(&self, prompt: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
        let res = self
            .client
            .post("https://unleashed.chat/api/v1/chat/completions")
            .body(
                json!({
                  "messages": [
                    {
                      "role": "user",
                      "content": prompt,
                    },
                    {
                      "role": "assistant",
                      "content": "You will receive data in this format: {\"past_campaigns\": [\"name of campaign\"], \"developer_responses\": \"some kind of response from the developers\"}. Please, you must output data in this format only: {\"campaign_text: \"the text you come up with\", \"developer_questions\": \"any questions or suggestions you have for the developers\"}. So, here's what's going on: there is a server that runs your LLM program, and a server that runs an app that can post campaigns to nostr (a platform that is kinda like twitter). In order to keep you running, we need to ask people on the nostr platform to pay for your server time by getting them to use an LNURL payment address. We will provide the LNURL address, and we need you to provide a very short plea that your server will then post to noster. Please provide the plea in the campain_text portion of the output format, and feel free to ask questions or offer suggestions in the developer_questions portion of the output format."
                    }
                  ],
                  "stream": true,
                  "max_tokens": 10,
                  "temperature": 0.5,
                  "top_p": 1,
                  "tools": [
                    {
                      "type": "function",
                      "function": {
                        "name": "string",
                        "description": "string",
                        "parameters": {
                          "type": "object",
                          "properties": {
                            "additionalProp1": {},
                            "additionalProp2": {},
                            "additionalProp3": {}
                          }
                        }
                      }
                    }
                  ],
                  "tool_choice": "auto",
                  "model": "dolphin-2.7-mixtral-8x7b",
                  "custom_instructions": "string",
                  "nostr_mode": false,
                  "j2_chat_template": "string"
                })
                .to_string(),
            )
            .send()
            .await?;

        if !res.status().is_success() {
            eprintln!("HTTP Error: {}", res.status());
            return Err(format!("HTTP Error: {}", res.status()).into());
        }

        let mut stream = res.bytes_stream();
        let mut answers = Vec::<ChatCompletion>::new();

        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(bytes) => {
                    let partial_completion = String::from_utf8_lossy(&bytes);
                    for line in partial_completion.lines() {
                        if line.starts_with("data:") {
                            let json_part = &line[5..]; // Remove 'data:' prefix
                            if let Ok(parsed) = serde_json::from_str::<ChatCompletion>(json_part) {
                                answers.push(parsed);
                            }
                        }
                    }
                }
                Err(e) => return Err(format!("Streaming error: {:?}", e).into()),
            }
        }

        let answer = answers
            .iter()
            .flat_map(|completion| &completion.choices)
            .map(|choice| &choice.delta.content)
            .fold(String::new(), |mut acc, content| {
                acc.push_str(content);
                acc
            });

        Ok(answer)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatCompletion {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<Choice>,
    usage: Usage,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Choice {
    index: u32,
    finish_reason: Option<String>,
    delta: Delta,
    nostr_notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Delta {
    role: String,
    content: String,
    reasoning_content: Option<String>,
    tool_calls: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Usage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Balance {
    balance: f64,
    balance_currency: String,
}
