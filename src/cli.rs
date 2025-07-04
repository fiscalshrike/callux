use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "callux")]
#[command(about = "A fast calendar agenda utility for Waybar and Hyprland")]
#[command(version = "0.1.0")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Show calendar agenda")]
    Agenda {
        #[arg(short, long, value_enum, default_value = "human")]
        format: OutputFormat,
        #[arg(short, long, help = "Number of events to show")]
        limit: Option<usize>,
        #[arg(short, long, help = "Days to look ahead")]
        days: Option<i64>,
    },
    #[command(about = "List available calendars")]
    ListCalendars,
    #[command(about = "Configure the application")]
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
    #[command(about = "Authenticate with Google Calendar")]
    Auth,
}

#[derive(Subcommand)]
pub enum ConfigAction {
    #[command(about = "Show current configuration")]
    Show,
    #[command(about = "Set a configuration value")]
    Set {
        #[arg(help = "Configuration key")]
        key: String,
        #[arg(help = "Configuration value")]
        value: String,
    },
    #[command(about = "Initialize default configuration")]
    Init,
}

#[derive(clap::ValueEnum, Clone)]
pub enum OutputFormat {
    #[value(name = "json")]
    Json,
    #[value(name = "human")]
    Human,
    #[value(name = "colored")]
    Colored,
}