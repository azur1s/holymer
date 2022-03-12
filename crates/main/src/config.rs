use serde_derive::Deserialize;
use toml::{from_str, de::Error};

#[derive(Deserialize)]
pub struct Config {
    pub compiler: CompilerOptions,
}

#[derive(Deserialize)]
pub struct CompilerOptions {
    pub compiler: String,
}

pub fn default_config() -> Config {
    Config {
        compiler: CompilerOptions {
            compiler: "clang++".to_string(),
        },
    }
}

pub fn parse_config(toml_content: &str) -> Result<Config, Error> {
    let config: Config = from_str(toml_content)?;
    Ok(config)
}