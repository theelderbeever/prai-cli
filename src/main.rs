use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use lazy_static::lazy_static;
use log::{debug, info};

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
    #[arg(default_value = "HEAD")]
    commit1: String,
    commit2: Option<String>,

    #[arg(short, long, default_value = ":!*.lock")]
    exclude: String,

    /// The provider profile to use for generation. Will default to the value in the config default.
    #[arg(short, long, global = true)]
    profile: Option<String>,

    /// Path to config file for sourcing providers
    #[arg(short = 'f', long = "config", global = true, default_value = default_config_string())]
    config: PathBuf,

    /// Generate a PR title instead of description
    #[arg(long)]
    title: bool,

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

    println!("{}", description);

    Ok(())
}
