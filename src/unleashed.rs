use futures_util::StreamExt;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
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

    pub async fn ask_llm(&self, prompt: &str) -> Result<String, Box<dyn Error>> {
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
                      "content": "There is a server that runs The LLM and also a server that runs The Program which queries the LLM and can also post advertising campaings to the internet. The advertising campaings help fund The LLM server and The Program server. We need to ask humans on the internet for Bitcoin and Lightning payments in order to keep the servers running. You will receive a consolidated list of previous campaigns, a list of current platforms to post advertisements, and you are to generate the text for a next campaign which is intended to bring in money to keep your servers running."
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
        let mut buffer = Vec::new();
        let mut answer = String::new();

        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(bytes) => {
                    let chunk_str = String::from_utf8_lossy(&bytes);
                    println!("Received chunk: {}", chunk_str);
                    buffer.extend_from_slice(&bytes);
                }
                Err(e) => return Err(format!("Streaming error: {:?}", e).into()),
            }
        }

        let streamed_data = String::from_utf8(buffer)?;

        if let Ok(parsed) = serde_json::from_str::<ChatCompletion>(&streamed_data) {
            println!("parsed: {:?}", &parsed);
            let partial_answer = parsed
                .choices
                .iter()
                .fold(String::new(), |mut acc, choice| {
                    acc.push_str(&choice.delta.content);
                    acc
                });
            answer.push_str(&partial_answer);
        }

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
