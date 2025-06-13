use log::debug;

use crate::{providers::Provider, settings::OllamaSettings};

pub struct OllamaProvider {
    config: OllamaSettings,
}

impl Provider for OllamaProvider {
    type Config = OllamaSettings;
    fn from_config(config: Self::Config) -> Self {
        debug!("Create provider from {:?}", config);
        Self { config }
    }
    fn build_url(&self) -> String {
        format!("{}/api/generate", self.config.url)
    }

    fn build_request_body(&self, prompt: &str) -> anyhow::Result<serde_json::Value> {
        Ok(serde_json::json!({
            "model": self.config.model,
            "prompt": prompt,
            "stream": false,
            "options": {
                "temperature": self.config.temperature,
                "top_p": self.config.top_p,
                "num_predict": self.config.num_predict
            }
        }))
    }

    fn parse_response(&self, response: serde_json::Value) -> anyhow::Result<String> {
        let generated_text = response
            .get("response")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        Ok(generated_text)
    }
}
