use std::process::Command;

use anyhow::{Result, anyhow};
use indoc::indoc;

pub struct Prompt;

impl Prompt {
    pub const DEFAULT_ROLE: &str = indoc! {
        r#"You are a technical writer creating PR summaries."#
    };
    pub const DEFAULT_TEMPLATE: &str = indoc! {
        r#"# Summary
        Brief description of what this PR accomplishes.

        ## Changes Made
        - List the main changes
        - Use bullet points for clarity
        - Be specific about what was modified

        ## Type of Change
        Feature, Bug, Chore, Docs


        ## Breaking Changes
        List any breaking changes and migration steps if applicable.

        ## Additional Notes
        Any additional context, considerations, or follow-up items."#

    };
    pub const DEFAULT_DIRECTIVE: &str = indoc! {
        r#"Write a professional summary of the changes made in the diff. Start directly with the summary, no conversational preamble.
        Use markdown syntax. Mention any breaking changes. Do not write code. Use the following as an example template. Do not check boxes which are not
        included in the diff."#
    };

    pub const DEFAULT_TITLE_DIRECTIVE: &str = indoc! {
        r#"Analyze this git diff and generate a concise PR title. The title should:
        - Be 50 characters or less
        - Use imperative mood (e.g., "Add feature" not "Added feature")
        - Be specific but concise about the main change
        - Not include punctuation at the end
        Don't include your own thought process. The output should be just the PR title."#
    };
    pub fn render(
        base: &str,
        head: &str,
        exclude: &[&str],
        role: Option<&str>,
        directive: Option<&str>,
        template: Option<&str>,
        is_title: bool,
    ) -> Result<String> {
        let default_directive = if is_title {
            Self::DEFAULT_TITLE_DIRECTIVE
        } else {
            Self::DEFAULT_DIRECTIVE
        };

        Ok(format!(
            indoc! {"
            [ROLE]
            {role}
            [DIRECTIVE]
            {directive}
            [PULL_REQUEST_TEMPLATE]
            {template}
            [DIFF]
            {diff}
            "},
            diff = Self::get_git_diff(base, head, exclude)?,
            role = role.unwrap_or(Self::DEFAULT_ROLE),
            directive = directive.unwrap_or(default_directive),
            template = template.unwrap_or(Self::DEFAULT_TEMPLATE)
        ))
    }

    fn get_git_diff(base: &str, head: &str, exclude: &[&str]) -> Result<String> {
        let mut cmd = Command::new("git");

        cmd.arg("diff").arg(base).arg(head);

        let output = cmd.args(["--", &exclude.join(" ")]).output()?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Git diff failed: {}", error);
        }

        let diff = String::from_utf8(output.stdout)?;

        if diff.is_empty() {
            return Err(anyhow!("No differences between `{base}` and `{head}`"));
        }

        Ok(diff)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_str_eq;

    use crate::providers::prompt::Prompt;

    #[test]
    fn test_prompt() {
        let prompt = Prompt::render("683ddd6", "d2bbcc5", &[":!*.lock"], None, None, None, false)
            .unwrap()
            .replace(" \n", "\n");

        assert_str_eq!(EXPECTED.trim(), prompt.as_str().trim());
    }

    const EXPECTED: &str = indoc! {
        r#"[ROLE]
        You are a technical writer creating PR summaries.
        [DIRECTIVE]
        Write a professional summary of the changes made in the diff. Start directly with the summary, no conversational preamble.
        Use markdown syntax. Mention any breaking changes. Do not write code. Use the following as an example template. Do not check boxes which are not
        included in the diff.
        [PULL_REQUEST_TEMPLATE]
        # Summary
        Brief description of what this PR accomplishes.

        ## Changes Made
        - List the main changes
        - Use bullet points for clarity
        - Be specific about what was modified

        ## Type of Change
        Feature, Bug, Chore, Docs


        ## Breaking Changes
        List any breaking changes and migration steps if applicable.

        ## Additional Notes
        Any additional context, considerations, or follow-up items.
        [DIFF]
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
                "#
    };
}
