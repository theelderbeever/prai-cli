# prai - AI-Powered PR Description Generator

A command-line tool that generates concise pull request descriptions from git diffs using Anthropic's Claude AI.

## Overview

`prai` analyzes the differences between two git commits and automatically generates a professional PR description highlighting:
- What changes were made
- Why these changes matter
- Any breaking changes or important notes

Perfect for streamlining your PR workflow and ensuring consistent, informative descriptions.

## Installation

### Via Cargo

```bash
cargo install --git https://github.com/theelderbeever/prai-cli.git
```

### From Source

```bash
git clone https://github.com/theelderbeever/prai-cli.git
cd prai-cli/prai
cargo install --path .
```

## Setup

You'll need an Anthropic API key to use this tool. Get one from [Anthropic's Console](https://console.anthropic.com/).

Set your API key as an environment variable:

```bash
export ANTHROPIC_API_KEY="your-api-key-here"
```

Or pass it directly via the `--api-key` flag.

## Usage

This works great with `git-fzf.sh` so I would recommend using that. Otherwise this ultimately is just calling `git diff` under the hood so whatever works for the commit arguments there should work here too.

```bash
prai <base-commit> <head-commit> [OPTIONS]
```

### Options

- `--exclude, -e`: Files to exclude from diff (default: `:!*.lock`)
- `--api-key, -a`: Anthropic API key (or use `ANTHROPIC_API_KEY` env var)

### Examples

Generate a PR description comparing two commits:
```bash
prai main feature-branch
```

Compare specific commit hashes:
```bash
prai abc123 def456
```

Exclude additional files:
```bash
prai main HEAD --exclude ":!*.lock :!dist/"
```

Pass API key directly:
```bash
prai main HEAD --api-key sk-ant-...
```

## Sample Output

```
## Changes Made
• Implemented user authentication with JWT tokens
• Added password validation and hashing utilities
• Created login/logout API endpoints

## Impact
• Enables secure user sessions across the application
• Provides foundation for role-based access control

## Notes
• Breaking change: `/api/login` now requires email instead of username
• New dependency: `jsonwebtoken` crate added
```

## Requirements

- Git (for generating diffs)
- Rust 1.70+ (for installation)
- Anthropic API key

## TODO

- [ ] Support for additional model providers (OpenAI, Google, Ollama, etc.)
- [ ] Configurable prompt templates

## License

MIT
