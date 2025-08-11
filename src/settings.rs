use std::path::Path;

use anyhow::{Result, anyhow};
use config::{ConfigBuilder, FileFormat, builder::DefaultState};
use secrecy::SecretString;
use serde::{Deserialize, Serialize};

type DefaultConfigBuilder = ConfigBuilder<DefaultState>;

#[derive(Debug, Deserialize, Serialize)]
pub struct Settings {
    pub default: String,
    #[serde(rename = "profile")]
    profiles: Vec<Profile>,
}

impl Settings {
    pub fn get(self, profile: Option<String>) -> Result<Profile> {
        let name = profile.unwrap_or(self.default);

        self.profiles
            .into_iter()
            .find(|p| p.name.eq(&name))
            .ok_or(anyhow!("Unable to find profile `{name}`"))
    }
    pub fn from_path(path: &Path) -> anyhow::Result<Self> {
        Ok(Self::builder(path)?.build()?.try_deserialize()?)
    }
    pub fn builder(path: &Path) -> anyhow::Result<DefaultConfigBuilder> {
        Ok(config::Config::builder()
            .add_source(config::File::new(path.to_str().unwrap(), FileFormat::Toml))
            .add_source(
                config::Environment::default()
                    .prefix("PRAI")
                    .prefix_separator("__")
                    .separator("__"),
            ))
    }
}

pub fn serialize_secret<S>(_value: &SecretString, s: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    s.serialize_str("[REDACTED]")
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Profile {
    pub name: String,
    pub role: Option<String>,
    pub directive: Option<String>,
    #[serde(flatten)]
    pub provider: Provider,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "provider", rename_all = "lowercase")]
pub enum Provider {
    Anthropic(AnthropicSettings),
    Ollama(OllamaSettings),
    OpenAI(OpenAISettings),
    Google(GoogleSettings),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AnthropicSettings {
    #[serde(default = "AnthropicSettings::default_version")]
    pub version: String,
    pub model: String,
    #[serde(serialize_with = "serialize_secret")]
    pub api_key: SecretString,
    #[serde(default = "AnthropicSettings::default_max_tokens")]
    pub max_tokens: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
}

impl AnthropicSettings {
    fn default_version() -> String {
        String::from("2023-06-01")
    }

    fn default_max_tokens() -> u32 {
        500
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OllamaSettings {
    #[serde(default = "OllamaSettings::default_url")]
    pub url: String,
    pub model: String,
    #[serde(default = "OllamaSettings::default_temperature")]
    pub temperature: f32,
    #[serde(default = "OllamaSettings::default_top_p")]
    pub top_p: f32,
    #[serde(default = "OllamaSettings::default_num_predict")]
    pub num_predict: u32,
}

impl OllamaSettings {
    fn default_url() -> String {
        String::from("http://localhost:11434")
    }

    fn default_temperature() -> f32 {
        0.3
    }

    fn default_top_p() -> f32 {
        0.9
    }

    fn default_num_predict() -> u32 {
        500
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OpenAISettings {
    pub model: String,
    #[serde(serialize_with = "serialize_secret")]
    pub api_key: SecretString,
    #[serde(default = "OpenAISettings::default_base_url")]
    pub base_url: String,
    #[serde(default = "OpenAISettings::default_max_tokens")]
    pub max_tokens: u32,
    #[serde(default = "OpenAISettings::default_temperature")]
    pub temperature: f32,
    #[serde(default = "OpenAISettings::default_top_p")]
    pub top_p: f32,
    #[serde(default = "OpenAISettings::default_frequency_penalty")]
    pub frequency_penalty: f32,
    #[serde(default = "OpenAISettings::default_presence_penalty")]
    pub presence_penalty: f32,
}

impl OpenAISettings {
    fn default_base_url() -> String {
        String::from("https://api.openai.com/v1")
    }

    fn default_max_tokens() -> u32 {
        500
    }

    fn default_temperature() -> f32 {
        0.3
    }

    fn default_top_p() -> f32 {
        0.9
    }

    fn default_frequency_penalty() -> f32 {
        0.0
    }

    fn default_presence_penalty() -> f32 {
        0.0
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GoogleSettings {
    pub model: String,
    #[serde(serialize_with = "serialize_secret")]
    pub api_key: SecretString,
    #[serde(default = "GoogleSettings::default_base_url")]
    pub base_url: String,
    #[serde(default = "GoogleSettings::default_max_tokens")]
    pub max_tokens: u32,
    #[serde(default = "GoogleSettings::default_temperature")]
    pub temperature: f32,
    #[serde(default = "GoogleSettings::default_top_p")]
    pub top_p: f32,
}

impl GoogleSettings {
    fn default_base_url() -> String {
        String::from("https://generativelanguage.googleapis.com/v1beta")
    }

    fn default_max_tokens() -> u32 {
        500
    }

    fn default_temperature() -> f32 {
        0.3
    }

    fn default_top_p() -> f32 {
        0.9
    }
}
