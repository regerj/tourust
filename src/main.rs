use std::{io, thread::sleep, time::Duration};

use app::App;
use ratatui::{
    Terminal,
    crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
        execute,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    },
    prelude::CrosstermBackend,
};

mod app;
mod tui;

fn main() {
    enable_raw_mode().expect("Enabling raw mode");
    let mut stderr = io::stderr();
    execute!(stderr, EnterAlternateScreen, EnableMouseCapture).unwrap();

    let backend = CrosstermBackend::new(stderr);
    let mut terminal = Terminal::new(backend).expect("creating new terminal");

    // create app and run it
    let mut app = App::new();

    loop {
        terminal.draw(|f| tui::ui(f, &mut app)).expect("Failed to draw");
        if let Event::Key(key) = event::read().expect("Reading event") {
            if key.kind == event::KeyEventKind::Release {
                continue;
            }

            match key.code {
                KeyCode::Esc => break,
                KeyCode::Char(ch) => app.input.push(ch),
                KeyCode::Up => app.search_result_state.select_previous(),
                KeyCode::Down => app.search_result_state.select_next(),
                KeyCode::Backspace => {
                    app.input.pop();
                }
                _ => {}
            }
        }
        sleep(Duration::from_millis(100));
    }

    disable_raw_mode().expect("Disabling raw mode");
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )
    .expect("Post steps");
    terminal.show_cursor().expect("Showing cursor");
}
