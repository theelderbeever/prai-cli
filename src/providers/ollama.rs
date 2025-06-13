use crate::{
    Prompt,
    providers::{Provider, Request},
    settings::OllamaSettings,
};

pub struct OllamaProvider {
    client: reqwest::blocking::Client,
    config: OllamaSettings,
}

impl Provider for OllamaProvider {
    type Config = OllamaSettings;
    fn from_config(config: Self::Config) -> Self {
        let client = reqwest::blocking::Client::new();
        Self { client, config }
    }
    fn make_request(&self, request: Request) -> anyhow::Result<String> {
        let prompt = Prompt::render(
            request.base.as_str(),
            request.head.as_deref(),
            request.exclude.as_str(),
            request.role.as_deref(),
            request.directive.as_deref(),
        )?;

        let url = format!("{}/api/generate", self.config.url);

        let request_body = serde_json::json!({
            "model": self.config.model,
            "prompt": prompt,
            "stream": false,
            "options": {
                "temperature": self.config.temperature,
                "top_p": self.config.top_p,
                "num_predict": self.config.num_predict
            }
        });

        let response = self.client.post(&url).json(&request_body).send()?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Ollama API request failed with status: {}",
                response.status()
            ));
        }

        let response_json: serde_json::Value = response.json()?;

        let generated_text = response_json
            .get("response")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        Ok(generated_text)
    }
}
