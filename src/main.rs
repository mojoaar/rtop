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
#[command(name = "rtop", about = "A beautiful terminal system monitor")]
struct Cli {
    #[arg(long)]
    theme: Option<String>,
    #[arg(long)]
    interval: Option<u64>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = config::Config::load()?
        .with_theme(cli.theme)
        .with_interval(cli.interval);
    app::run(&config)
}
