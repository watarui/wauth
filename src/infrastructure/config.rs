use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize)]
pub struct Config {
    pub aws_profile: String,
    pub dynamodb_table_name: String,
}

impl Config {
    pub fn load() -> Result<Self> {
        // 開発モードかどうかをチェック（CARGO_MANIFESTの存在で判断）
        if Self::is_development_mode() {
            // 開発モードの場合は.envから読み込み
            // println!("Running in development mode, using .env file");
            dotenvy::dotenv().ok();

            return Ok(Self {
                aws_profile: std::env::var("AWS_PROFILE")
                    .context("AWS_PROFILE must be set in environment variables or .env file")?,
                dynamodb_table_name: std::env::var("DYNAMODB_TABLE_NAME").context(
                    "DYNAMODB_TABLE_NAME must be set in environment variables or .env file",
                )?,
            });
        }

        // 本番モードの場合はconfig.tomlから読み込み
        // println!("Running in production mode, using config.toml");
        let config_path = Self::get_config_path()?;
        let content = std::fs::read_to_string(&config_path).context(format!(
            "Failed to read config file at: {}",
            config_path.display()
        ))?;
        let config: Config = toml::from_str(&content).context("Failed to parse config.toml")?;

        Ok(config)
    }

    fn is_development_mode() -> bool {
        // WAUTH_DEV環境変数での明示的な指定
        if let Ok(dev_mode) = std::env::var("WAUTH_DEV") {
            return dev_mode.to_lowercase() == "true";
        }

        // カレントディレクトリまたはその親ディレクトリにCargo.tomlが存在するかチェック
        let mut current_dir = std::env::current_dir().unwrap_or_default();

        loop {
            let cargo_toml = current_dir.join("Cargo.toml");
            if cargo_toml.exists() {
                return true;
            }

            if !current_dir.pop() {
                break;
            }
        }

        false
    }

    fn get_config_path() -> Result<PathBuf> {
        let home =
            dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
        let config_dir = home.join(".config").join("wauth");
        Ok(config_dir.join("config.toml"))
    }
}
