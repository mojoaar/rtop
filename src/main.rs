use anyhow::Result;
use clap::Parser;

mod app;
mod config;
mod data;
mod event;
mod platform;
mod theme;
mod ui;

#[derive(Parser)]
#[command(name = "rtop", version, about = "A beautiful terminal system monitor")]
struct Cli {
    #[arg(
        long,
        help = "Theme: latte, frappe, macchiato, mocha, dracula, nord, github-dark"
    )]
    theme: Option<String>,
    #[arg(long, help = "Refresh interval in milliseconds")]
    interval: Option<u64>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = config::Config::load()?
        .with_theme(cli.theme)
        .with_interval(cli.interval);
    app::run(&config)
}
