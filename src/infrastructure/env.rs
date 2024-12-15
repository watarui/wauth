use anyhow::{bail, Context, Result};

pub fn get_required_env_var(key: &str) -> Result<String> {
    let value = std::env::var(key).context(format!(
        "{} must be set in environment variables or .env file",
        key
    ))?;

    if value.trim().is_empty() {
        bail!("{} cannot be empty", key);
    }

    Ok(value)
}
