pub mod prompt;

use anyhow::Result;

pub trait Provider {
    type Config;

    fn from_config(config: Self::Config) -> Self;
    fn make_request(&self, diff: &str) -> Result<String>;
}
