use log::debug;

use crate::{providers::Provider, settings::AnthropicSettings};

pub struct AnthropicProvider {
    config: AnthropicSettings,
}

impl Provider for AnthropicProvider {
    type Config = AnthropicSettings;

    fn from_config(config: Self::Config) -> Self {
        debug!("Create Anthropic provider from {:?}", config);
        Self { config }
    }

    fn build_url(&self) -> String {
        "https://api.anthropic.com/v1/messages".to_string()
    }

    fn build_request_body(&self, prompt: &str) -> anyhow::Result<serde_json::Value> {
        Ok(serde_json::json!({
            "model": self.config.model,
            "max_tokens": self.config.max_tokens,
            "temperature": self.config.temperature,
            "top_p": self.config.top_p,
            "messages": [
                {
                    "role": "user",
                    "content": prompt
                }
            ]
        }))
    }

    fn parse_response(&self, response: serde_json::Value) -> anyhow::Result<String> {
        let generated_text = response
            .get("content")
            .and_then(|content| content.as_array())
            .and_then(|content_array| content_array.first())
            .and_then(|first_content| first_content.get("text"))
            .and_then(|text| text.as_str())
            .unwrap_or("")
            .to_string();

        Ok(generated_text)
    }

    fn get_client(&self) -> reqwest::blocking::Client {
        let mut headers = reqwest::header::HeaderMap::new();

        if let Ok(api_key_header) = reqwest::header::HeaderValue::from_str(&self.config.api_key) {
            headers.insert("x-api-key", api_key_header);
        }

        if let Ok(version_header) = reqwest::header::HeaderValue::from_str(&self.config.version) {
            headers.insert("anthropic-version", version_header);
        }

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
