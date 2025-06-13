pub mod anthropic;
pub mod google;
pub mod ollama;
pub mod openai;
pub mod prompt;

use anyhow::Result;
use bon::Builder;

#[derive(Builder)]
pub struct Request {
    pub base: String,
    pub head: Option<String>,
    pub exclude: String,
    pub role: Option<String>,
    pub directive: Option<String>,
}

pub trait Provider {
    type Config;

    fn from_config(config: Self::Config) -> Self;
    fn make_request(&self, request: Request) -> Result<String>;
}
