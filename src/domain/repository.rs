use super::models::TOTPEntry;
use async_trait::async_trait;

#[async_trait]
pub trait TOTPRepository {
    async fn save_secret(&self, site_name: String, secret: String) -> Result<(), anyhow::Error>;
    async fn delete_secret(&self, site_name: &str) -> Result<(), anyhow::Error>;
    async fn get_secret(&self, site_name: &str) -> Result<Option<TOTPEntry>, anyhow::Error>;
    async fn list_sites(&self) -> Result<Vec<String>, anyhow::Error>;
}
