use log::debug;
use secrecy::ExposeSecret;

use crate::{providers::Provider, settings::GoogleSettings};

pub struct GoogleProvider {
    config: GoogleSettings,
}

impl Provider for GoogleProvider {
    type Config = GoogleSettings;

    fn from_config(config: Self::Config) -> Self {
        debug!("Create Google provider from {:?}", config);
        Self { config }
    }

    fn build_url(&self) -> String {
        format!(
            "{}/models/{}:generateContent?key={}",
            self.config.base_url,
            self.config.model,
            self.config.api_key.expose_secret()
        )
    }

    fn build_request_body(&self, prompt: &str) -> anyhow::Result<serde_json::Value> {
        Ok(serde_json::json!({
            "contents": [
                {
                    "parts": [
                        {
                            "text": prompt
                        }
                    ]
                }
            ],
            "generationConfig": {
                "temperature": self.config.temperature,
                "topP": self.config.top_p,
                "maxOutputTokens": self.config.max_tokens
            }
        }))
    }

    fn parse_response(&self, response: serde_json::Value) -> anyhow::Result<String> {
        let generated_text = response
            .get("candidates")
            .and_then(|candidates| candidates.as_array())
            .and_then(|candidates| candidates.first())
            .and_then(|candidate| candidate.get("content"))
            .and_then(|content| content.get("parts"))
            .and_then(|parts| parts.as_array())
            .and_then(|parts| parts.first())
            .and_then(|part| part.get("text"))
            .and_then(|text| text.as_str())
            .unwrap_or("")
            .to_string();

        Ok(generated_text)
    }

    fn get_client(&self) -> reqwest::blocking::Client {
        let mut headers = reqwest::header::HeaderMap::new();

        headers.insert(
            reqwest::header::CONTENT_TYPE,
            reqwest::header::HeaderValue::from_static("application/json"),
        );

        reqwest::blocking::Client::builder()
            .default_headers(headers)
            .build()
            .unwrap_or_else(|_| reqwest::blocking::Client::new())
    }
}
