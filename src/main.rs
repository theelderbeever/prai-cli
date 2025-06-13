use anyhow::Result;
use clap::Parser;
use indoc::indoc;
use log::{debug, info, trace, warn};
use reqwest::{
    blocking::Client,
    header::{CONTENT_TYPE, HeaderMap, HeaderValue},
};
use serde_json::json;

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

    /// Verbose mode (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}

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

    // Generate PR description using Anthropic API
    let description = generate_pr_description(&args)?;

    println!("{}", description);

    Ok(())
}

/// Generate PR description using Anthropic API
///
/// # Arguments
/// * `args` - CLI arguments containing commit hashes, exclude patterns, and API key
fn generate_pr_description(args: &Args) -> Result<String> {
    info!("Generating PR description using Anthropic API");

    let client = Client::new();
    let mut headers = HeaderMap::new();
    headers.insert("x-api-key", HeaderValue::from_str(&args.api_key).unwrap());
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert("anthropic-version", HeaderValue::from_static("2023-06-01"));

    debug!("Building prompt for git diff analysis");
    let prompt = prai::Prompt::builder()
        .role("You are a senior Rust engineer".to_string())
        .prompt(indoc!{
            r#"Analyze this git diff and create a concise PR description. Focus on:
            - What changes were made (be specific but brief)
            - Why these changes matter
            - Any breaking changes or important notes
            Keep it under 150 words and use bullet points for clarity. Don't include implementation details unless critical.
            Don't unclude your own thought process. The output should be just the content of the PR summary."#
        }.to_string()
        ).build().render(&args.commit1, args.commit2.as_deref(), &args.exclude)?;

    trace!("Prompt content:\n{}", prompt);

    let request_body = json!({
        "model": "claude-3-sonnet-20240229",
        "max_tokens": 500,
        "messages": [{
            "role": "user",
            "content": prompt
        }]
    });

    debug!("Sending request to Anthropic API");
    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .headers(headers)
        .json(&request_body)
        .send()?;

    let status = response.status();
    if !status.is_success() {
        let error_text = response.text()?;
        warn!("API request failed with status: {}", status);
        anyhow::bail!("API request failed: {}", error_text);
    }

    debug!("Parsing API response");
    let response_json: serde_json::Value = response.json()?;

    let content = response_json["content"][0]["text"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Invalid API response format"))?;

    info!("Successfully generated PR description");
    Ok(content.to_string())
}
