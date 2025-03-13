use std::{io, path::{Path, PathBuf}, thread::sleep, time::Duration};

use app::{App, Ref};
use clap::Parser;
use cli::Cli;
use error::{Error, Result};
use fuzzy_matcher::clangd::fuzzy_match;
use nvim_rs::Value;
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
mod error;
mod tui;
mod cli;

#[derive(Clone)]
struct NvimHandler {}

impl nvim_rs::Handler for NvimHandler {
    type Writer = nvim_rs::compat::tokio::Compat<tokio::io::WriteHalf<tokio::net::UnixStream>>;
}

async fn select_callback(socket: PathBuf, selection: Ref) -> Result<()> {
    let handler = NvimHandler{};

    let (nvim, _) = nvim_rs::create::tokio::new_path(socket, handler).await.unwrap_or_else(|err| {
        println!("Error occured: {}", err);
        panic!()
    });

    let perr = async |msg: String| {
        nvim.echo(vec![Value::Array(vec![msg.into()])], true, Vec::new()).await
    };

    let wins = nvim.list_wins().await?;

    for win in &wins {
        perr(format!("Win: {}", win.get_buf().await?.get_name().await?)).await?;
    }

    let win = match nvim.get_current_win().await {
        Ok(win) => win,
        Err(err) => {
            perr(format!("Error occurred: {}", err)).await?;
            panic!()
        }
    };

    perr(format!("Curr buffer: {}", win.get_buf().await?.get_name().await?)).await?;

    if let Err(err) = wins[0].set_cursor((selection.line as i64, selection.column as i64)).await {
        perr(format!("Error occurred: {}", err)).await?;
        panic!()
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    enable_raw_mode()?;
    let mut stderr = io::stderr();
    execute!(stderr, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stderr);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let mut app = App::new()?;
    if let Some(cmd) = cli.command {
        match cmd {
            cli::Command::Nvim(args) => {
                app.select_callback = Some(Box::new(move |x| select_callback(args.socket.clone(), x)));
            }
        }
    }

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
                    app.search_results = app
                        .refs
                        .iter()
                        .filter_map(|elem| {
                            fuzzy_match(&elem.sig, &app.input).map(|prio| (elem.to_owned(), prio))
                        })
                        .collect()
                }
                KeyCode::Up => app.search_result_state.select_previous(),
                KeyCode::Down => app.search_result_state.select_next(),
                KeyCode::Backspace => {
                    app.input.pop();
                    app.search_results = app
                        .refs
                        .iter()
                        .map(|elem| {
                            (
                                elem.to_owned(),
                                fuzzy_match(&elem.sig, &app.input).unwrap_or_default(),
                            )
                        })
                        .collect();
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
                    println!(
                        "Source File: {}, Line: {}, Column: {}",
                        selected_result.file.display(),
                        selected_result.line,
                        selected_result.column
                    );

                    if let Some(callback) = app.select_callback {
                        callback.call(selected_result.clone()).await?;
                    }
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
