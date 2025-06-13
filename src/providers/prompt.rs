use std::process::Command;

use anyhow::Result;
use indoc::indoc;

pub struct Prompt;

impl Prompt {
    pub const DEFAULT_ROLE: &str = "You are a senior engineer";
    pub const DEFAULT_DIRECTIVE: &str = indoc! {
        r#"Analyze this git diff and create a concise PR description. Focus on:
        - What changes were made (be specific but brief)
        - Why these changes matter
        - Any breaking changes or important notes
        Keep it under 150 words and use bullet points for clarity. Don't include implementation details unless critical.
        Don't unclude your own thought process. The output should be just the content of the PR summary."#
    };
    pub fn render(
        base: &str,
        head: Option<&str>,
        exclude: &str,
        role: Option<&str>,
        directive: Option<&str>,
    ) -> Result<String> {
        Ok(format!(
            indoc! {"[CONTEXT]
            {diff}
            [ROLE]
            {role}
            [DIRECTIVE]
            {directive}"},
            diff = Self::get_git_diff(base, head, exclude)?,
            role = role.unwrap_or(Self::DEFAULT_ROLE),
            directive = directive.unwrap_or(Self::DEFAULT_DIRECTIVE)
        ))
    }

    fn get_git_diff(base: &str, head: Option<&str>, exclude: &str) -> Result<String> {
        let mut cmd = Command::new("git");

        cmd.arg("diff").arg(base);

        if let Some(head) = head {
            cmd.arg(head);
        }

        let output = cmd.args(["--", exclude]).output()?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Git diff failed: {}", error);
        }

        Ok(String::from_utf8(output.stdout)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_str_eq;

    use crate::providers::prompt::Prompt;

    #[test]
    fn test_prompt() {
        let prompt = Prompt::render("683ddd6", Some("d2bbcc5"), ":!*.lock", None, None)
            .unwrap()
            .replace(" \n", "\n");

        assert_str_eq!(EXPECTED, prompt.as_str());
    }

    const EXPECTED: &str = indoc! {
        r#"[CONTEXT]
        diff --git a/src/main.rs b/src/main.rs
        index 88f8b72..935c050 100644
        --- a/src/main.rs
        +++ b/src/main.rs
        @@ -1,5 +1,5 @@
         use clap::Parser;
        -use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};
        +use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderValue};
         use serde_json::json;
         use std::process::Command;

        @@ -13,6 +13,9 @@ struct Args {
             /// Head commit hash
             head: String,

        +    #[arg(short, long, default_value = ":!*.lock")]
        +    exclude: String,
        +
             /// Anthropic API key (or set ANTHROPIC_API_KEY env var)
             #[arg(short, long, env = "ANTHROPIC_API_KEY")]
             api_key: String,
        @@ -23,7 +26,9 @@ async fn main() -> Result<(), Box<dyn std::error::Error>> {
             let args = Args::parse();

             // Generate git diff
        -    let diff = get_git_diff(&args.base, &args.head)?;
        +    let diff = get_git_diff(&args.base, &args.head, &args.exclude)?;
        +
        +    println!("{diff}");

             if diff.trim().is_empty() {
                 println!(
        @@ -41,8 +46,15 @@ async fn main() -> Result<(), Box<dyn std::error::Error>> {
             Ok(())
         }

        -fn get_git_diff(base: &str, head: &str) -> Result<String, Box<dyn std::error::Error>> {
        -    let output = Command::new("git").args(["diff", base, head]).output()?;
        +fn get_git_diff(
        +    base: &str,
        +    head: &str,
        +    exclude: &str,
        +) -> Result<String, Box<dyn std::error::Error>> {
        +    println!("{exclude}");
        +    let output = Command::new("git")
        +        .args(["diff", base, head, "--", exclude])
        +        .output()?;

             if !output.status.success() {
                 let error = String::from_utf8_lossy(&output.stderr);
        @@ -57,12 +69,8 @@ async fn generate_pr_description(
             api_key: &str,
         ) -> Result<String, Box<dyn std::error::Error>> {
             let client = reqwest::Client::new();
        -
             let mut headers = HeaderMap::new();
        -    headers.insert(
        -        AUTHORIZATION,
        -        HeaderValue::from_str(&format!("Bearer {}", api_key))?,
        -    );
        +    headers.insert("x-api-key", HeaderValue::from_str(api_key).unwrap());
             headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
             headers.insert("anthropic-version", HeaderValue::from_static("2023-06-01"));

        @@ -90,6 +98,7 @@ async fn generate_pr_description(
             let response = client
                 .post("https://api.anthropic.com/v1/messages")
                 .headers(headers)
        +        // .bearer_auth(api_key)
                 .json(&request_body)
                 .send()
                 .await?;

        [ROLE]
        You are a senior Rust engineer
        [DIRECTIVE]
        Analyze this git diff and create a concise PR description. Focus on:
        - What changes were made (be specific but brief)
        - Why these changes matter
        - Any breaking changes or important notes
        Keep it under 150 words and use bullet points for clarity. Don't include implementation details unless critical.
        Don't unclude your own thought process. The output should be just the content of the PR summary."#
    };
}
