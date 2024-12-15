use clap::{Parser, Subcommand};
mod application;
mod domain;
mod infrastructure;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Site name to generate code for
    site_name: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new TOTP secret
    Add {
        /// Site name (e.g., github, google)
        site_name: String,
        /// Secret key
        secret: String,
    },
    /// Delete a TOTP secret
    Delete {
        /// Site name to delete
        site_name: String,
    },
    /// List all registered sites
    List,
    /// Generate fish shell completion script
    GenerateFishCompletion,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let app = application::TOTPApplication::new().await?;

    match &cli.command {
        Some(Commands::Add { site_name, secret }) => {
            app.add_secret(site_name.clone(), secret.clone()).await?;
            println!("Added secret for {}", site_name);
        }
        Some(Commands::Delete { site_name }) => {
            app.delete_secret(site_name).await?;
            println!("Deleted secret for {}", site_name);
        }
        Some(Commands::List) => {
            let sites = app.list_sites().await?;
            println!("Registered sites:");
            for site in sites {
                println!("- {}", site);
            }
        }
        Some(Commands::GenerateFishCompletion) => {
            app.generate_fish_completion().await?;
        }
        None => {
            if let Some(site_name) = cli.site_name {
                app.show_code_for_site(&site_name).await?;
            } else {
                println!("Please provide a site name or use --help for available commands");
            }
        }
    }

    Ok(())
}
