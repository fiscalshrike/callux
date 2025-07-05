mod auth;
mod cache;
mod calendar;
mod cli;
mod config;
mod error;
mod output;

use crate::auth::AuthManager;
use crate::calendar::CalendarClient;
use crate::cli::{Cli, Commands, ConfigAction};
use crate::config::Config;
use crate::output::OutputFormatter;
use clap::Parser;
use colored::*;
use rustls::crypto::ring::default_provider;

#[tokio::main]
async fn main() {
    default_provider()
        .install_default()
        .expect("Failed to install crypto provider");
    let cli = Cli::parse();

    match run(cli).await {
        Ok(()) => {}
        Err(e) => {
            eprintln!("{}: {}", "Error".red().bold(), e);
            std::process::exit(1);
        }
    }
}

async fn run(cli: Cli) -> anyhow::Result<()> {
    match cli.command {
        Commands::Agenda {
            format,
            limit,
            days,
        } => {
            let config = Config::load()?;
            let client = CalendarClient::new(config.clone());
            let days_ahead = days.unwrap_or(7);
            let event_limit = limit.or(Some(config.display.max_events));

            let events = client
                .get_events(days_ahead, event_limit)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to get events: {}", e))?;

            let formatter = OutputFormatter::new(
                format,
                config.display.date_format,
                config.display.max_events,
            );

            let output = formatter.format_events(&events);
            println!("{}", output);
        }
        Commands::ListCalendars => {
            let config = Config::load()?;
            let client = CalendarClient::new(config);

            let calendars = client
                .list_calendars()
                .await
                .map_err(|e| anyhow::anyhow!("Failed to list calendars: {}", e))?;

            println!("{}", "Available Calendars:".bright_blue().bold());
            for calendar in calendars {
                let id = calendar.id.as_deref().unwrap_or("unknown");
                let name = calendar.summary.as_deref().unwrap_or("Untitled");
                let primary = if calendar.primary.unwrap_or(false) {
                    " (primary)"
                } else {
                    ""
                };
                println!(
                    "  {}: {}{}",
                    id.bright_green(),
                    name,
                    primary.bright_yellow()
                );
            }
        }
        Commands::Config { action } => match action {
            ConfigAction::Show => {
                let config = Config::load()?;
                let config_str = toml::to_string_pretty(&config)?;
                println!("{}", config_str);
            }
            ConfigAction::Set { key, value } => {
                println!("Setting configuration is not yet implemented");
                println!("Key: {}, Value: {}", key, value);
            }
            ConfigAction::Init => {
                let config = Config::default();
                config.save()?;

                let auth_manager = AuthManager::new(config.clone());
                auth_manager.create_sample_credentials()?;

                println!("{}", "Configuration initialized!".bright_green().bold());
                println!("Please edit the following files:");
                println!(
                    "1. Configuration: {}",
                    Config::load()?
                        .expand_path("~/.config/callux/config.toml")
                        .bright_yellow()
                );
                println!(
                    "2. Credentials: {}",
                    config
                        .expand_path(&config.auth.credentials_path)
                        .bright_yellow()
                );
                println!("\nTo get Google Calendar credentials:");
                println!("1. Go to https://console.developers.google.com/");
                println!("2. Create a new project or select an existing one");
                println!("3. Enable the Google Calendar API");
                println!("4. Create OAuth 2.0 credentials");
                println!("5. Download the credentials JSON file");
                println!("6. Replace the placeholder values in the credentials file");
            }
        },
        Commands::Auth => {
            let config = Config::load()?;
            let auth_manager = AuthManager::new(config);

            match auth_manager.get_token().await {
                Ok(_) => {
                    println!("{}", "Authentication successful!".bright_green().bold());
                    println!("You can now use callux to access your calendar.");
                }
                Err(e) => {
                    eprintln!("{}: {}", "Authentication failed".red().bold(), e);
                    println!("\nTo authenticate:");
                    println!("1. Run 'callux config init' to set up configuration");
                    println!("2. Add your Google Calendar credentials");
                    println!("3. Run 'callux auth' again");
                    std::process::exit(1);
                }
            }
        }
    }

    Ok(())
}
