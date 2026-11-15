mod app;
mod config;
mod model;
mod sound;
mod timer;
mod ui;

use app::App;
use ratatui::crossterm::event::{self, Event, KeyEventKind};
use std::{io, time::Duration};

fn main() -> io::Result<()> {
    let mut app = App::new();

    let mut terminal = ratatui::init(); // raw mode + alt screen + panic hook
    let result = run(&mut terminal, &mut app);
    ratatui::restore();

    app.persist_on_exit();
    result
}

fn run(terminal: &mut ratatui::DefaultTerminal, app: &mut App) -> io::Result<()> {
    while !app.should_quit {
        app.tick();
        terminal.draw(|f| ui::draw(f, app))?;

        if event::poll(Duration::from_millis(100))? {
            // リサイズは次のループで自動的に再描画される
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    app.handle_key(key);
                }
            }
        }
    }
    Ok(())
}
