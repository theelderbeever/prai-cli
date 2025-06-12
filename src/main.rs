use clap::Parser;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};
use serde_json::json;
use std::process::Command;

#[derive(Parser)]
#[command(name = "git-pr-desc")]
#[command(about = "Generate PR descriptions from git diffs using Anthropic's API")]
struct Args {
    /// Base commit hash
    base: String,

    /// Head commit hash
    head: String,

    /// Anthropic API key (or set ANTHROPIC_API_KEY env var)
    #[arg(short, long, env = "ANTHROPIC_API_KEY")]
    api_key: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Generate git diff
    let diff = get_git_diff(&args.base, &args.head)?;

    if diff.trim().is_empty() {
        println!(
            "No differences found between {} and {}",
            args.base, args.head
        );
        return Ok(());
    }

    // Generate PR description using Anthropic API
    let description = generate_pr_description(&diff, &args.api_key).await?;

    println!("{}", description);

    Ok(())
}

fn get_git_diff(base: &str, head: &str) -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("git").args(["diff", base, head]).output()?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Git diff failed: {}", error).into());
    }

    Ok(String::from_utf8(output.stdout)?)
}

async fn generate_pr_description(
    diff: &str,
    api_key: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", api_key))?,
    );
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert("anthropic-version", HeaderValue::from_static("2023-06-01"));

    let prompt = format!(
        "Analyze this git diff and create a concise PR description. Focus on:\n\
        - What changes were made (be specific but brief)\n\
        - Why these changes matter\n\
        - Any breaking changes or important notes\n\
        \n\
        Keep it under 150 words and use bullet points for clarity. Don't include implementation details unless critical.\n\
        \n\
        Git diff:\n```\n{}\n```",
        diff
    );

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
        .send()
        .await?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        return Err(format!("API request failed: {}", error_text).into());
    }

    let response_json: serde_json::Value = response.json().await?;

    let content = response_json["content"][0]["text"]
        .as_str()
        .ok_or("Invalid API response format")?;

    Ok(content.to_string())
}
