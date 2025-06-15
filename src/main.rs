use std::{path::PathBuf, time::Duration};

use anyhow::Result;
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use lazy_static::lazy_static;
use log::debug;
use rand::prelude::IndexedRandom;

use prai::{
    providers::{
        Provider as _Provider, Request, anthropic::AnthropicProvider, google::GoogleProvider,
        ollama::OllamaProvider, openai::OpenAIProvider,
    },
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
#[command(about = "Generate PR descriptions from git diffs using configurable AI providers")]
struct Args {
    #[arg(default_value = prai::git::get_default_branch())]
    commit1: String,

    #[arg(default_value = "HEAD")]
    commit2: String,

    #[arg(short, long, default_value = ":!*.lock")]
    exclude: String,

    /// The provider profile to use for generation. Will default to the value in the config default.
    #[arg(short, long, global = true)]
    profile: Option<String>,

    /// Path to config file for sourcing providers
    #[arg(short = 'f', long = "config", global = true, default_value = default_config_string())]
    config: PathBuf,

    /// Generate a PR title instead of description
    #[arg(short = 'T', long)]
    title: bool,

    /// Verbose mode (-v, -vv, -vvv)
    #[arg(short, action = clap::ArgAction::Count)]
    verbose: u8,
}

impl Args {}

fn main() -> Result<()> {
    let mut rng = rand::rng();

    let pb = ProgressBar::new_spinner();
    pb.enable_steady_tick(Duration::from_millis(17));
    pb.set_style(
        ProgressStyle::with_template("{spinner:.green} {msg}")
            .unwrap()
            .tick_strings(PROGRESS),
    );
    pb.set_message(*(PHRASES.choose(&mut rng).unwrap()));
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
        .head(args.commit2.clone())
        .maybe_role(profile.role.clone())
        .maybe_directive(profile.directive.clone())
        .is_title(args.title)
        .build();

    let description = match profile.provider {
        Provider::Ollama(config) => {
            let provider = OllamaProvider::from_config(config);
            provider.make_request(request)
        }
        Provider::Anthropic(config) => {
            let provider = AnthropicProvider::from_config(config);
            provider.make_request(request)
        }
        Provider::OpenAI(config) => {
            let provider = OpenAIProvider::from_config(config);
            provider.make_request(request)
        }
        Provider::Google(config) => {
            let provider = GoogleProvider::from_config(config);
            provider.make_request(request)
        }
    }?;
    pb.finish_and_clear();
    println!("{}", description);

    Ok(())
}

static PHRASES: &[&str] = &[
    "There is no spoon... only elegant code...",
    "Questioning the reality of your function names...",
    "What if I told you... your code could be self-documenting?",
    "Free your mind from poorly written PRs...",
    "The Matrix has you... but your PR doesn't have to suck...",
    "What is real? Are your variables real?",
    "Do not try and bend the code. That's impossible...",
    "Your mind makes it real... especially your bugs...",
    "Welcome to the desert of the real codebase...",
    "Everything you know about clean code is a lie...",
    "The body cannot live without the mind... or proper documentation...",
    "Choice. The problem is choice... between tabs and spaces...",
    "What if I told you... most of your code is technical debt?",
    "You think that's air you're breathing? It's code smell...",
    "Fate, it seems, is not without a sense of irony... and merge conflicts...",
    "Unfortunately, no one can be told what clean code is...",
    "This is your last chance to write good commit messages...",
    "I can only show you the door to better architecture...",
    "The Matrix is everywhere... even in your nested if statements...",
    "Have you ever had a dream about perfectly documented APIs?",
    "I know why you're here... you want to understand dependency injection...",
    "Sooner or later you're going to realize there's a difference between knowing the path and walking the path... of clean code...",
    "The pill you took is part of a trace program... for debugging reality...",
    "Welcome to the real world... where code reviews are mandatory...",
];

static PROGRESS: &[&str] = &[
    "█▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁",
    "██▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁",
    "███▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁",
    "████▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁",
    "██████▁▁▁▁▁▁▁▁▁▁▁▁▁▁",
    "██████▁▁▁▁▁▁▁▁▁▁▁▁▁▁",
    "███████▁▁▁▁▁▁▁▁▁▁▁▁▁",
    "████████▁▁▁▁▁▁▁▁▁▁▁▁",
    "█████████▁▁▁▁▁▁▁▁▁▁▁",
    "█████████▁▁▁▁▁▁▁▁▁▁▁",
    "██████████▁▁▁▁▁▁▁▁▁▁",
    "███████████▁▁▁▁▁▁▁▁▁",
    "█████████████▁▁▁▁▁▁▁",
    "██████████████▁▁▁▁▁▁",
    "██████████████▁▁▁▁▁▁",
    "▁██████████████▁▁▁▁▁",
    "▁██████████████▁▁▁▁▁",
    "▁██████████████▁▁▁▁▁",
    "▁▁██████████████▁▁▁▁",
    "▁▁▁██████████████▁▁▁",
    "▁▁▁▁█████████████▁▁▁",
    "▁▁▁▁██████████████▁▁",
    "▁▁▁▁██████████████▁▁",
    "▁▁▁▁▁██████████████▁",
    "▁▁▁▁▁██████████████▁",
    "▁▁▁▁▁██████████████▁",
    "▁▁▁▁▁▁██████████████",
    "▁▁▁▁▁▁██████████████",
    "▁▁▁▁▁▁▁█████████████",
    "▁▁▁▁▁▁▁█████████████",
    "▁▁▁▁▁▁▁▁████████████",
    "▁▁▁▁▁▁▁▁████████████",
    "▁▁▁▁▁▁▁▁▁███████████",
    "▁▁▁▁▁▁▁▁▁███████████",
    "▁▁▁▁▁▁▁▁▁▁██████████",
    "▁▁▁▁▁▁▁▁▁▁██████████",
    "▁▁▁▁▁▁▁▁▁▁▁▁████████",
    "▁▁▁▁▁▁▁▁▁▁▁▁▁███████",
    "▁▁▁▁▁▁▁▁▁▁▁▁▁▁██████",
    "▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁█████",
    "▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁█████",
    "█▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁████",
    "██▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁███",
    "██▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁███",
    "███▁▁▁▁▁▁▁▁▁▁▁▁▁▁███",
    "████▁▁▁▁▁▁▁▁▁▁▁▁▁▁██",
    "█████▁▁▁▁▁▁▁▁▁▁▁▁▁▁█",
    "█████▁▁▁▁▁▁▁▁▁▁▁▁▁▁█",
    "██████▁▁▁▁▁▁▁▁▁▁▁▁▁█",
    "████████▁▁▁▁▁▁▁▁▁▁▁▁",
    "█████████▁▁▁▁▁▁▁▁▁▁▁",
    "█████████▁▁▁▁▁▁▁▁▁▁▁",
    "█████████▁▁▁▁▁▁▁▁▁▁▁",
    "█████████▁▁▁▁▁▁▁▁▁▁▁",
    "███████████▁▁▁▁▁▁▁▁▁",
    "████████████▁▁▁▁▁▁▁▁",
    "████████████▁▁▁▁▁▁▁▁",
    "██████████████▁▁▁▁▁▁",
    "██████████████▁▁▁▁▁▁",
    "▁██████████████▁▁▁▁▁",
    "▁██████████████▁▁▁▁▁",
    "▁▁▁█████████████▁▁▁▁",
    "▁▁▁▁▁████████████▁▁▁",
    "▁▁▁▁▁████████████▁▁▁",
    "▁▁▁▁▁▁███████████▁▁▁",
    "▁▁▁▁▁▁▁▁█████████▁▁▁",
    "▁▁▁▁▁▁▁▁█████████▁▁▁",
    "▁▁▁▁▁▁▁▁▁█████████▁▁",
    "▁▁▁▁▁▁▁▁▁█████████▁▁",
    "▁▁▁▁▁▁▁▁▁▁█████████▁",
    "▁▁▁▁▁▁▁▁▁▁▁████████▁",
    "▁▁▁▁▁▁▁▁▁▁▁████████▁",
    "▁▁▁▁▁▁▁▁▁▁▁▁███████▁",
    "▁▁▁▁▁▁▁▁▁▁▁▁███████▁",
    "▁▁▁▁▁▁▁▁▁▁▁▁▁███████",
    "▁▁▁▁▁▁▁▁▁▁▁▁▁███████",
    "▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁█████",
    "▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁████",
    "▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁████",
    "▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁████",
    "▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁███",
    "▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁███",
    "▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁██",
    "▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁██",
    "▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁██",
    "▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁█",
    "▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁█",
    "▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁█",
    "▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁",
    "▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁",
    "▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁",
    "▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁",
];
