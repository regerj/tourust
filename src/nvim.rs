use std::{path::PathBuf, usize};

use futures::StreamExt;
use log::{debug, error};
use nvim_rs::{compat::tokio::Compat, Buffer, Neovim, Value, Window};
use ratatui::crossterm::event::{self, Event, KeyCode};
use tokio::{io::WriteHalf, net::UnixStream};

use crate::{
    app::Ref,
    error::{Error, Result},
};

#[derive(Clone)]
struct NvimHandler {}

impl nvim_rs::Handler for NvimHandler {
    type Writer = nvim_rs::compat::tokio::Compat<tokio::io::WriteHalf<tokio::net::UnixStream>>;
}

pub async fn select_callback(socket: PathBuf, selection: Ref) -> Result<()> {
    let handler = NvimHandler {};
    debug!("selection: {:?}", selection);

    // Get our API
    let (nvim, _) = nvim_rs::create::tokio::new_path(socket, handler)
        .await
        .unwrap_or_else(|err| {
            println!("Error occured: {}", err);
            panic!()
        });

    let self_win = nvim.get_current_win().await?;
    self_win.hide().await?;

    // Select the proper window
    let wins = nvim.list_wins().await?;

    //select_window(&nvim).await?;

    for buf in nvim.list_bufs().await? {
        debug!(
            "Examining buffer: {} against match: {}",
            buf.get_name().await?,
            selection.file.display()
        );
        if buf.get_name().await? == selection.file.to_str().ok_or(Error::Utf8)? {
            debug!("Found match!");
            debug!("Buflisted: {}", buf.get_option("buflisted").await?);
            buf.set_option("buflisted", Value::Boolean(true)).await?;
            nvim.get_current_win().await?.set_buf(&buf).await?;
            //wins[0].set_buf(&buf).await?;
            break;
        }
    }

    if let Err(err) = nvim.get_current_win().await?
        .set_cursor((selection.line as i64, selection.column as i64))
        .await
    {
        error!("Failed to set cursor: {}", err.to_string());
        panic!()
    }
    Ok(())
}

async fn find_or_open_buf(nvim: &Neovim<Compat<WriteHalf<UnixStream>>>, selection: &Ref) -> Result<Buffer<Compat<WriteHalf<UnixStream>>>> {
    for buf in nvim.list_bufs().await? {
        if buf.get_name().await? == selection.file.to_str().ok_or(Error::Utf8)? {
            debug!("Found existing buffer that matches requested");
            buf.set_option("buflisted", Value::Boolean(true)).await?;
            return Ok(buf);
        }
    }
    debug!("Did not find existing buffer that matches requested");
    let prev_buf = nvim.get_current_buf().await?;
    let buf = nvim.create_buf(true, false).await?;
    nvim.set_current_buf(&buf).await?;
    nvim.command(format!("edit {}", selection.file.display()).as_str()).await?;
    nvim.set_current_buf(&prev_buf).await?;
    Ok(buf)
}

async fn select_window(nvim: &Neovim<Compat<WriteHalf<UnixStream>>>) -> Result<Window<Compat<WriteHalf<UnixStream>>>> {
    // Hide the current window
    let app_win = nvim.get_current_win().await?;
    debug!("app_win is: {}", app_win.get_buf().await?.get_name().await?);

    let mut config = Vec::new();
    config.push((
        Value::String("relative".into()),
        Value::String("editor".into()),
    ));
    config.push((Value::String("width".into()), Value::Integer(1.into())));
    config.push((
        Value::String("height".into()),
        Value::Integer(1.into()),
    ));
    config.push((
        Value::String("col".into()),
        Value::Integer(0.into()),
    ));
    config.push((
        Value::String("row".into()),
        Value::Integer(0.into()),
    ));
    config.push((Value::String("focusable".into()), Value::Boolean(true)));
    config.push((
        Value::String("style".into()),
        Value::String("minimal".into()),
    ));

    let invis_win = nvim.open_win(&app_win.get_buf().await?, true, config).await?;
    debug!("Opened invisible window to capture input");

    app_win.hide().await?;
    debug!("Hidden previous app window");

    let wins = nvim.list_wins().await?;

    struct SelectableWindow {
        window: Window<Compat<WriteHalf<UnixStream>>>,
        select_prompt: Window<Compat<WriteHalf<UnixStream>>>,
    }

    let mut selectable_wins: Vec<SelectableWindow> = Vec::new();

    for (i, win) in wins.iter().enumerate() {
        // Empty indicates a normal buffer. If it is any other kind of buffer, we will assume that
        // we cannot select this window for opening text files.
        if win.get_buf().await?.get_option("buftype").await? != Value::String("".into()) {
            continue;
        }

        // Create our buffer and gather some sizing information
        let buf = nvim.create_buf(false, true).await?;
        buf.set_lines(0, 0, false, vec![format!("   Pick Window {i}   ").into()])
            .await?;
        let width = 19;
        let height = 1;
        let center_x = win.get_width().await? / 2;
        let center_y = win.get_height().await? / 2;

        // Configure our buffer
        let mut config = Vec::new();
        config.push((
            Value::String("relative".into()),
            Value::String("win".into()),
        ));
        config.push((Value::String("win".into()), win.get_value().clone()));
        config.push((Value::String("width".into()), Value::Integer(width.into())));
        config.push((
            Value::String("height".into()),
            Value::Integer(height.into()),
        ));
        config.push((
            Value::String("col".into()),
            Value::Integer((center_x - width / 2).into()),
        ));
        config.push((
            Value::String("row".into()),
            Value::Integer((center_y - height / 2).into()),
        ));
        config.push((Value::String("focusable".into()), Value::Boolean(false)));
        config.push((
            Value::String("style".into()),
            Value::String("minimal".into()),
        ));
        let border = vec![
            Value::String("╭".into()),
            Value::String("─".into()),
            Value::String("╮".into()),
            Value::String("│".into()),
            Value::String("╯".into()),
            Value::String("─".into()),
            Value::String("╰".into()),
            Value::String("│".into()),
        ];
        config.push((Value::String("border".into()), Value::Array(border)));

        // Open a new floating window in the center of the existing text editing window with our
        // hello window
        let prompt = nvim.open_win(&buf, false, config).await?;
        selectable_wins.push(SelectableWindow { window: win.clone(), select_prompt: prompt });
    }

    // Loop to gather input
    loop {
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Release {
                continue;
            }
            match key.code {
                KeyCode::Char(ch) => {
                    if let Some(num) = ch.to_digit(10) {
                        for selectable_win in &selectable_wins {
                            selectable_win.select_prompt.close(false).await?;
                        }

                        return Ok(selectable_wins[num as usize].window.clone());
                    }
                }
                _ => continue,
            }
        }
    }
}
