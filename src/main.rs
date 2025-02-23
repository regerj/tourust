use std::{io, thread::sleep, time::Duration};

use app::App;
use error::Error;
use fuzzy_matcher::clangd::fuzzy_match;
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
mod error;

fn main() -> Result<(), Error> {
    enable_raw_mode()?;
    let mut stderr = io::stderr();
    execute!(stderr, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stderr);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let mut app = App::new()?;

    loop {
        terminal.draw(|f| tui::ui(f, &mut app))?;
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Release {
                continue;
            }

            match key.code {
                KeyCode::Esc => break,
                KeyCode::Char(ch) => {
                    // Every time the user types a character, add it to the input, drain the refs,
                    // map them to assign new prio, then collect and reassign them to refs.
                    app.input.push(ch);
                    app.search_results = app.refs.iter().filter_map(|elem| {
                        fuzzy_match(&elem.sig, &app.input).map(|prio| (elem.to_owned(), prio))
                    }).collect()
                }
                KeyCode::Up => app.search_result_state.select_previous(),
                KeyCode::Down => app.search_result_state.select_next(),
                KeyCode::Backspace => {
                    app.input.pop();
                    app.search_results = app.refs.iter().map(|elem| (elem.to_owned(), fuzzy_match(&elem.sig, &app.input).unwrap_or_default())).collect();
                }
                KeyCode::Enter => {
                    // Continue if nothing is selected
                    let i = if let Some(i) = app.search_result_state.selected() {
                        i
                    } else {
                        continue;
                    };

                    // Get the selected item, close the TUI, print info, and exit
                    let search_results = app.search_results.to_owned().into_sorted_vec();
                    let selected_result = &search_results[i];
                    disable_raw_mode()?;
                    execute!(
                        terminal.backend_mut(),
                        LeaveAlternateScreen,
                        DisableMouseCapture
                    )?;
                    terminal.show_cursor()?;               
                    println!("Source File: {}, Line: {}, Column: {}", selected_result.file.display(), selected_result.line, selected_result.column);
                    return Ok(());
                }
                _ => {}
            }
        }
        sleep(Duration::from_millis(25));
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}
