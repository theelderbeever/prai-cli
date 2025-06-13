use anyhow::Result;
use clap::Parser;
use indoc::indoc;
use reqwest::{
    blocking::Client,
    header::{CONTENT_TYPE, HeaderMap, HeaderValue},
};
use serde_json::json;

#[derive(Parser)]
#[command(name = "prai")]
#[command(about = "Generate PR descriptions from git diffs using Anthropic's API")]
struct Args {
    /// Base commit hash
    #[arg(default_value = "HEAD")]
    commit1: String,

    /// Head commit hash
    commit2: Option<String>,

    #[arg(short, long, default_value = ":!*.lock")]
    exclude: String,

    /// Anthropic API key (or set ANTHROPIC_API_KEY env var)
    #[arg(short, long, env = "ANTHROPIC_API_KEY")]
    api_key: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Generate PR description using Anthropic API
    let description = generate_pr_description(&args.api_key)?;

    println!("{}", description);

    Ok(())
}

fn generate_pr_description(api_key: &str) -> Result<String> {
    let client = Client::new();
    let mut headers = HeaderMap::new();
    headers.insert("x-api-key", HeaderValue::from_str(api_key).unwrap());
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert("anthropic-version", HeaderValue::from_static("2023-06-01"));

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
        ).build().render("683ddd6", Some("d2bbcc5"), ":!*.lock")?;

    let request_body = json!({
        "model": "claude-3-sonnet-20240229",
        "max_tokens": 500,
        "messages": [{
            "role": "user",
            "content": prompt
        }]
    });

    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .headers(headers)
        .json(&request_body)
        .send()?;

    if !response.status().is_success() {
        let error_text = response.text()?;
        anyhow::bail!("API request failed: {}", error_text);
    }

    let response_json: serde_json::Value = response.json()?;

    let content = response_json["content"][0]["text"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Invalid API response format"))?;

    Ok(content.to_string())
}
