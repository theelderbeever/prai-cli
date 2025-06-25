pub mod anthropic;
pub mod google;
pub mod ollama;
pub mod openai;
pub mod prompt;

use anyhow::Result;
use bon::Builder;
use log::trace;

#[derive(Builder, Debug)]
pub struct Request {
    pub base: String,
    pub head: String,
    pub exclude: Vec<String>,
    pub template: Option<String>,
    pub role: Option<String>,
    pub directive: Option<String>,
    pub is_title: bool,
}

pub trait Provider {
    type Config;

    fn from_config(config: Self::Config) -> Self;

    /// Build the prompt from the request parameters
    fn build_prompt(&self, request: &Request) -> Result<String> {
        let exclusions: Vec<&str> = request.exclude.iter().map(|s| s.as_str()).collect();
        let prompt = prompt::Prompt::render(
            request.base.as_str(),
            request.head.as_str(),
            &exclusions,
            request.role.as_deref(),
            request.directive.as_deref(),
            request.template.as_deref(),
            request.is_title,
        )?;
        trace!("Prompt:\n{prompt}");
        Ok(prompt)
    }

    /// Build the API endpoint URL
    fn build_url(&self) -> String;

    /// Build the request body for the API call
    fn build_request_body(&self, prompt: &str) -> Result<serde_json::Value>;

    /// Parse the response and extract the generated text
    fn parse_response(&self, response: serde_json::Value) -> Result<String>;

    /// Get the HTTP client (default implementation creates a new blocking client)
    fn get_client(&self) -> reqwest::blocking::Client {
        reqwest::blocking::Client::new()
    }

    /// Make the HTTP request (default implementation)
    fn make_http_request(&self, url: &str, body: &serde_json::Value) -> Result<serde_json::Value> {
        let client = self.get_client();
        let response = client.post(url).json(body).send()?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "API request failed with status: {}",
                response.status()
            ));
        }

        Ok(response.json()?)
    }

    /// Main request method with default implementation using the other trait methods
    fn make_request(&self, request: Request) -> Result<String> {
        trace!("{:?}", request);

        let prompt = self.build_prompt(&request)?;
        let url = self.build_url();
        let request_body = self.build_request_body(&prompt)?;
        let response_json = self.make_http_request(&url, &request_body)?;
        let generated_text = self.parse_response(response_json)?;

        Ok(generated_text)
    }
}
