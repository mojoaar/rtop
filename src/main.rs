use anyhow::Result;
use clap::Parser;
use crossterm::event::{self, Event, KeyCode};
use ratatui::widgets::Paragraph;

mod config;
mod data;
mod platform;
mod theme;

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

    let mut terminal = ratatui::try_init()?;
    let result = run(&mut terminal);
    ratatui::restore();
    let _ = config;
    result
}

fn run(terminal: &mut ratatui::DefaultTerminal) -> Result<()> {
    loop {
        terminal.draw(|frame| {
            frame.render_widget(Paragraph::new("rtop"), frame.area());
        })?;
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }
    }
    Ok(())
}
