use crate::domain::repository::TOTPRepository;
use crate::domain::totp::Totp;
use crate::infrastructure::aws::DynamoDBRepository;

pub struct TOTPApplication {
    repository: DynamoDBRepository,
}

impl TOTPApplication {
    pub async fn new() -> Result<Self, anyhow::Error> {
        Ok(Self {
            repository: DynamoDBRepository::new().await?,
        })
    }

    pub async fn add_secret(&self, site_name: String, secret: String) -> Result<(), anyhow::Error> {
        self.repository.save_secret(site_name, secret).await
    }

    pub async fn delete_secret(&self, site_name: &str) -> Result<(), anyhow::Error> {
        self.repository.delete_secret(site_name).await
    }

    pub async fn list_sites(&self) -> Result<Vec<String>, anyhow::Error> {
        self.repository.list_sites().await
    }

    pub async fn show_code_for_site(&self, site_name: &str) -> Result<(), anyhow::Error> {
        if let Some(entry) = self.repository.get_secret(site_name).await? {
            let totp = Totp::new(entry.secret);

            println!("WAUTH - TOTP Generator for {}", site_name);
            println!("---------------------");

            if let Ok(code) = totp.generate_code() {
                println!("Code: {} ({}s remaining)", code, Totp::remaining_seconds());
            }
        } else {
            println!("No secret found for site: {}", site_name);
        }

        Ok(())
    }

    pub async fn generate_fish_completion(&self) -> Result<(), anyhow::Error> {
        let sites = self.repository.list_sites().await?;

        // サブコマンドの補完
        println!("# Fish completion for wauth");
        println!("complete -f -c wauth -n \"__fish_use_subcommand\" -a \"add\" -d \"Add new TOTP secret for a site\"");
        println!("complete -f -c wauth -n \"__fish_use_subcommand\" -a \"delete\" -d \"Delete TOTP secret for a site\"");
        println!("complete -f -c wauth -n \"__fish_use_subcommand\" -a \"list\" -d \"List all registered sites\"");

        // サイト名の補完
        println!("complete -f -c wauth -n \"not __fish_seen_subcommand_from add delete list\" -a \"{}\" -d \"Site name\"",
            sites.join(" "));

        // delete コマンドのサイト名補完
        println!("complete -f -c wauth -n \"__fish_seen_subcommand_from delete\" -a \"{}\" -d \"Site to delete\"",
            sites.join(" "));

        // プロファイルオプションの補完
        println!("complete -f -c wauth -l profile -d \"Specify AWS profile\" -r");

        Ok(())
    }
}
