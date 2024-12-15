use crate::domain::models::TOTPEntry;
use crate::domain::repository::TOTPRepository;
use crate::infrastructure::config::Config;
use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use aws_config::profile::ProfileFileCredentialsProvider;
use aws_config::BehaviorVersion;
use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_dynamodb::Client;
use regex::Regex;

pub struct DynamoDBRepository {
    client: Client,
    table_name: String,
}

impl DynamoDBRepository {
    pub async fn new() -> Result<Self, anyhow::Error> {
        let config = Config::load()?;

        let credentials_provider = ProfileFileCredentialsProvider::builder()
            .profile_name(&config.aws_profile)
            .build();

        let aws_config = aws_config::defaults(BehaviorVersion::latest())
            .credentials_provider(credentials_provider)
            .load()
            .await;

        let client = Client::new(&aws_config);

        // println!("Using AWS profile: {}", config.aws_profile);
        // println!("Using DynamoDB table: {}", config.dynamodb_table_name);

        Ok(Self {
            client,
            table_name: config.dynamodb_table_name,
        })
    }
}

#[derive(Debug)]
struct ValidationError {
    field: String,
    message: String,
}

impl ValidationError {
    fn new(field: &str, message: &str) -> Self {
        Self {
            field: field.to_string(),
            message: message.to_string(),
        }
    }
}

use std::fmt;

// Display trait の実装
impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Validation error for {}: {}", self.field, self.message)
    }
}

// Error trait の実装
impl std::error::Error for ValidationError {}

#[async_trait]
impl TOTPRepository for DynamoDBRepository {
    async fn save_secret(&self, site_name: String, secret: String) -> Result<(), anyhow::Error> {
        // 基本的なバリデーション
        validate_site_name(&site_name)?;
        validate_secret(&secret)?;

        // 重複チェック
        self.check_site_name_uniqueness(&site_name).await?;

        self.client
            .put_item()
            .table_name(&self.table_name)
            .item(
                "site_name",
                aws_sdk_dynamodb::types::AttributeValue::S(site_name),
            )
            .item("secret", aws_sdk_dynamodb::types::AttributeValue::S(secret))
            .send()
            .await
            .context("Failed to save TOTP secret to DynamoDB")?;

        Ok(())
    }

    async fn delete_secret(&self, site_name: &str) -> Result<(), anyhow::Error> {
        self.client
            .delete_item()
            .table_name(&self.table_name)
            .key(
                "site_name",
                aws_sdk_dynamodb::types::AttributeValue::S(site_name.to_string()),
            )
            .send()
            .await?;

        Ok(())
    }

    async fn get_secret(&self, site_name: &str) -> Result<Option<TOTPEntry>, anyhow::Error> {
        let result = self
            .client
            .get_item()
            .table_name(&self.table_name)
            .key(
                "site_name",
                aws_sdk_dynamodb::types::AttributeValue::S(site_name.to_string()),
            )
            .send()
            .await?;

        Ok(result.item().map(|item| TOTPEntry {
            secret: item.get("secret").unwrap().as_s().unwrap().clone(),
        }))
    }

    async fn list_sites(&self) -> Result<Vec<String>, anyhow::Error> {
        let result = self
            .client
            .scan()
            .table_name(&self.table_name)
            .send()
            .await?;

        let sites = result
            .items()
            .iter()
            .filter_map(|item| {
                item.get("site_name")
                    .and_then(|av| av.as_s().ok())
                    .map(|s| s.to_string())
            })
            .collect();

        Ok(sites)
    }
}

impl DynamoDBRepository {
    async fn check_site_name_uniqueness(&self, site_name: &str) -> Result<()> {
        let result = self
            .client
            .get_item()
            .table_name(&self.table_name)
            .key("site_name", AttributeValue::S(site_name.to_string()))
            .consistent_read(true) // 強い整合性を持つ読み込みを使用
            .send()
            .await
            .context("Failed to check site name uniqueness")?;

        if result.item().is_some() {
            bail!(ValidationError::new(
                "site_name",
                &format!("Site name '{}' already exists", site_name)
            ));
        }

        Ok(())
    }
}

fn validate_site_name(site_name: &str) -> Result<()> {
    // 空文字チェック
    if site_name.trim().is_empty() {
        bail!(ValidationError::new(
            "site_name",
            "Site name cannot be empty"
        ));
    }

    // 長さチェック
    if site_name.len() > 100 {
        bail!(ValidationError::new(
            "site_name",
            "Site name must be 100 characters or less"
        ));
    }

    // 使用可能文字チェック
    let re = Regex::new(r"^[a-zA-Z0-9\-._]+$").unwrap();
    if !re.is_match(site_name) {
        bail!(ValidationError::new(
            "site_name",
            "Site name can only contain alphanumeric characters, hyphens, dots, and underscores"
        ));
    }

    Ok(())
}

fn validate_secret(secret: &str) -> Result<()> {
    // 空文字チェック
    if secret.trim().is_empty() {
        bail!(ValidationError::new("secret", "Secret cannot be empty"));
    }

    // Base32フォーマットチェック
    if !is_valid_base32(secret) {
        bail!(ValidationError::new(
            "secret",
            "Secret must be a valid Base32 string"
        ));
    }

    // 推奨される長さチェック（RFC 6238に基づく）
    if secret.len() < 16 {
        bail!(ValidationError::new(
            "secret",
            "Secret should be at least 16 characters long for security"
        ));
    }

    Ok(())
}

fn is_valid_base32(input: &str) -> bool {
    let base32_regex = Regex::new(r"^[A-Z2-7]+=*$").unwrap();
    base32_regex.is_match(input) && input.len() % 8 == 0
}
