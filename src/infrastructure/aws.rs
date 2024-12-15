use crate::domain::models::TOTPEntry;
use crate::domain::repository::TOTPRepository;
use crate::infrastructure::env::get_required_env_var;
use async_trait::async_trait;
use aws_config::profile::ProfileFileCredentialsProvider;
use aws_config::BehaviorVersion;
use aws_sdk_dynamodb::Client;

pub struct DynamoDBRepository {
    client: Client,
    table_name: String,
}

impl DynamoDBRepository {
    pub async fn new() -> Result<Self, anyhow::Error> {
        // .env ファイルを読み込む
        dotenvy::dotenv().ok();

        // 必須の環境変数をチェック
        let profile_name = get_required_env_var("AWS_PROFILE")?;
        let table_name = get_required_env_var("DYNAMODB_TABLE_NAME")?;

        let credentials_provider = ProfileFileCredentialsProvider::builder()
            .profile_name(&profile_name)
            .build();

        let config = aws_config::defaults(BehaviorVersion::latest())
            .credentials_provider(credentials_provider)
            .load()
            .await;

        let client = Client::new(&config);

        // println!("Using AWS profile: {}", profile_name);
        // println!("Using DynamoDB table: {}", table_name);

        Ok(Self { client, table_name })
    }
}

#[async_trait]
impl TOTPRepository for DynamoDBRepository {
    async fn save_secret(&self, site_name: String, secret: String) -> Result<(), anyhow::Error> {
        self.client
            .put_item()
            .table_name(&self.table_name)
            .item(
                "site_name",
                aws_sdk_dynamodb::types::AttributeValue::S(site_name),
            )
            .item("secret", aws_sdk_dynamodb::types::AttributeValue::S(secret))
            .send()
            .await?;

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