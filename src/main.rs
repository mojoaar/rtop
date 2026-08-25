use anyhow::Result;
use crossterm::event::{self, Event, KeyCode};
use ratatui::widgets::Paragraph;

fn main() -> Result<()> {
    let mut terminal = ratatui::try_init()?;
    let result = run(&mut terminal);
    ratatui::restore();
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
