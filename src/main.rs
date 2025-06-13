use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use lazy_static::lazy_static;
use log::{debug, info, trace, warn};
use reqwest::{
    blocking::Client,
    header::{CONTENT_TYPE, HeaderMap, HeaderValue},
};
use serde_json::json;

use prai::{
    Prompt,
    providers::{Provider as _Provider, Request, ollama::OllamaProvider},
    settings::{Provider, Settings},
};

fn default_config_string() -> &'static str {
    lazy_static! {
        static ref DEFAULT_PATH_STR: String = std::env::var("PRAI_HOME")
            .ok()
            .map(|s| PathBuf::from(s).join("config.toml"))
            .or(dirs::home_dir().map(|p| p.join(".config/prai/config.toml")))
            .map_or(String::from("./config.toml"), |p| p
                .to_string_lossy()
                .to_string());
    }
    &DEFAULT_PATH_STR
}

#[derive(Parser)]
#[command(name = "prai")]
#[command(about = "Generate PR descriptions from git diffs using Anthropic's API")]
struct Args {
    #[arg(default_value = "HEAD")]
    commit1: String,
    commit2: Option<String>,

    #[arg(short, long, default_value = ":!*.lock")]
    exclude: String,

    /// Anthropic API key (or set ANTHROPIC_API_KEY env var)
    #[arg(short, long, env = "ANTHROPIC_API_KEY")]
    api_key: String,

    /// The provider profile to use for generation. Will default to the value in the config default.
    #[arg(short, long, global = true)]
    profile: Option<String>,

    /// Path to config file for sourcing providers
    #[arg(short = 'f', long = "config", global = true, default_value = default_config_string())]
    config: PathBuf,

    /// Verbose mode (-v, -vv, -vvv)
    #[arg(short, action = clap::ArgAction::Count)]
    verbose: u8,
}

impl Args {}

fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging based on verbosity level
    let log_level = match args.verbose {
        0 => log::LevelFilter::Warn,
        1 => log::LevelFilter::Info,
        2 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    };

    env_logger::Builder::from_default_env()
        .filter_level(log_level)
        .init();

    info!("Starting prai with verbosity level: {}", args.verbose);
    debug!(
        "Using commit1: {}, commit2: {:?}",
        args.commit1, args.commit2
    );
    debug!("Exclude pattern: {}", args.exclude);

    let settings = Settings::from_path(&args.config)?;
    let profile = settings.get(args.profile.clone())?;

    let request = Request::builder()
        .base(args.commit1.clone())
        .exclude(args.exclude.clone())
        .maybe_head(args.commit2.clone())
        .maybe_role(profile.role.clone())
        .maybe_directive(profile.directive.clone())
        .build();

    let description = match profile.provider {
        Provider::Ollama(config) => {
            let provider = OllamaProvider::from_config(config);
            provider.make_request(request)
        }
        _ => todo!(),
    }?;

    println!("{}", description);

    Ok(())
}

// Generate PR description using Anthropic API

// # Arguments
// * `args` - CLI arguments containing commit hashes, exclude patterns, and API key
// fn generate_pr_description(args: &Args) -> Result<String> {
//     info!("Generating PR description using Anthropic API");

//     let client = Client::new();
//     let mut headers = HeaderMap::new();
//     headers.insert("x-api-key", HeaderValue::from_str(&args.api_key).unwrap());
//     headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
//     headers.insert("anthropic-version", HeaderValue::from_static("2023-06-01"));

//     debug!("Building prompt for git diff analysis");
//     let prompt = Prompt::render(
//         &args.commit1,
//         args.commit2.as_deref(),
//         &args.exclude,
//         None,
//         None,
//     )?;

//     trace!("Prompt content:\n{}", prompt);

//     let request_body = json!({
//         "model": "claude-3-sonnet-20240229",
//         "max_tokens": 500,
//         "messages": [{
//             "role": "user",
//             "content": prompt
//         }]
//     });

//     debug!("Sending request to Anthropic API");
//     let response = client
//         .post("https://api.anthropic.com/v1/messages")
//         .headers(headers)
//         .json(&request_body)
//         .send()?;

//     let status = response.status();
//     if !status.is_success() {
//         let error_text = response.text()?;
//         warn!("API request failed with status: {}", status);
//         anyhow::bail!("API request failed: {}", error_text);
//     }

//     debug!("Parsing API response");
//     let response_json: serde_json::Value = response.json()?;

//     let content = response_json["content"][0]["text"]
//         .as_str()
//         .ok_or_else(|| anyhow::anyhow!("Invalid API response format"))?;

//     info!("Successfully generated PR description");
//     Ok(content.to_string())
// }
