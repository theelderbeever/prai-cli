use log::debug;
use secrecy::ExposeSecret;

use crate::{providers::Provider, settings::OpenAISettings};

pub struct OpenAIProvider {
    config: OpenAISettings,
}

impl Provider for OpenAIProvider {
    type Config = OpenAISettings;

    fn from_config(config: Self::Config) -> Self {
        debug!("Create OpenAI provider from {config:?}");
        Self { config }
    }

    fn build_url(&self) -> String {
        format!("{}/chat/completions", self.config.base_url)
    }

    fn build_request_body(&self, prompt: &str) -> anyhow::Result<serde_json::Value> {
        Ok(serde_json::json!({
            "model": self.config.model,
            "messages": [
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "temperature": self.config.temperature,
            "top_p": self.config.top_p,
            "max_tokens": self.config.max_tokens,
            "stream": false
        }))
    }

    fn parse_response(&self, response: serde_json::Value) -> anyhow::Result<String> {
        let generated_text = response
            .get("choices")
            .and_then(|choices| choices.as_array())
            .and_then(|choices| choices.first())
            .and_then(|choice| choice.get("message"))
            .and_then(|message| message.get("content"))
            .and_then(|content| content.as_str())
            .unwrap_or("")
            .to_string();

        Ok(generated_text)
    }

    fn get_client(&self) -> reqwest::blocking::Client {
        let mut headers = reqwest::header::HeaderMap::new();

        if let Ok(auth_header) = reqwest::header::HeaderValue::from_str(&format!(
            "Bearer {}",
            self.config.api_key.expose_secret()
        )) {
            headers.insert(reqwest::header::AUTHORIZATION, auth_header);
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
