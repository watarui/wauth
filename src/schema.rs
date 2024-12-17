use crate::application::TOTPApplication;
use crate::domain::repository::TOTPRepository;
use async_graphql::{Context, Error, Object, Schema, SimpleObject};
use tracing::{error, info};

#[derive(SimpleObject)]
struct Site {
    name: String,
}

#[derive(SimpleObject)]
struct TOTPCode {
    code: String,
    remaining_seconds: u64,
    site_name: String,
}

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn list_sites(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<Site>> {
        let app = ctx.data::<TOTPApplication>()?;
        let sites = app.list_sites().await.map_err(|e| {
            error!("Failed to list sites: {}", e);
            Error::new(format!("Failed to list sites: {}", e))
        })?;

        info!("Retrieved {} sites", sites.len());
        Ok(sites.into_iter().map(|name| Site { name }).collect())
    }

    async fn get_totp_code(
        &self,
        ctx: &Context<'_>,
        site_name: String,
    ) -> async_graphql::Result<TOTPCode> {
        let app = ctx.data::<TOTPApplication>()?;

        let entry = app.repository.get_secret(&site_name).await.map_err(|e| {
            error!("Failed to get secret for site {}: {}", site_name, e);
            Error::new(format!("Failed to get TOTP code: {}", e))
        })?;

        let entry = entry.ok_or_else(|| Error::new("Site not found"))?;

        let totp = crate::domain::totp::Totp::new(entry.secret);
        let code = totp.generate_code().map_err(|e| {
            error!("Failed to generate TOTP code for site {}: {}", site_name, e);
            Error::new(format!("Failed to generate TOTP code: {}", e))
        })?;

        Ok(TOTPCode {
            code,
            remaining_seconds: crate::domain::totp::Totp::remaining_seconds(),
            site_name,
        })
    }
}

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    async fn add_site(
        &self,
        ctx: &Context<'_>,
        site_name: String,
        secret: String,
    ) -> async_graphql::Result<Site> {
        let app = ctx.data::<TOTPApplication>()?;

        app.add_secret(site_name.clone(), secret)
            .await
            .map_err(|e| {
                error!("Failed to add site {}: {}", site_name, e);
                Error::new(format!("Failed to add site: {}", e))
            })?;

        info!("Added new site: {}", site_name);
        Ok(Site { name: site_name })
    }

    async fn delete_site(
        &self,
        ctx: &Context<'_>,
        site_name: String,
    ) -> async_graphql::Result<Site> {
        let app = ctx.data::<TOTPApplication>()?;

        app.delete_secret(&site_name).await.map_err(|e| {
            error!("Failed to delete site {}: {}", site_name, e);
            Error::new(format!("Failed to delete site: {}", e))
        })?;

        info!("Deleted site: {}", site_name);
        Ok(Site { name: site_name })
    }
}

pub type AppSchema = Schema<QueryRoot, MutationRoot, async_graphql::EmptySubscription>;

pub fn build_schema(app: TOTPApplication) -> AppSchema {
    Schema::build(QueryRoot, MutationRoot, async_graphql::EmptySubscription)
        .data(app)
        .finish()
}
