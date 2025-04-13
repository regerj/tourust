use std::path::{Path, PathBuf};

use log::{debug, error};
use nvim_rs::{Buffer, Neovim, Value, Window, compat::tokio::Compat};
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

    // Hide our application window
    //let self_win = nvim.get_current_win().await?;
    //self_win.close(false).await?;

    let buf = find_or_open_buf(&nvim, &selection.file).await?;
    let win = find_text_win(&nvim).await?;
    win.set_buf(&buf).await?;
    //nvim.set_current_buf(&buf).await?;

    if let Err(err) = win
        .set_cursor((selection.line as i64, selection.column as i64))
        .await
    {
        error!("Failed to set cursor: {}", err.to_string());
        panic!()
    }
    Ok(())
}

async fn find_or_open_buf(
    nvim: &Neovim<Compat<WriteHalf<UnixStream>>>,
    file: &Path,
) -> Result<Buffer<Compat<WriteHalf<UnixStream>>>> {
    // Look for and return previous buffer if it matches
    for buf in nvim.list_bufs().await? {
        if buf.get_name().await? == file.to_str().ok_or(Error::Utf8)? {
            buf.set_option("buflisted", Value::Boolean(true)).await?;
            return Ok(buf);
        }
    }

    // We did not find a pre-existing buffer
    let prev_buf = nvim.get_current_buf().await?;
    let buf = nvim.create_buf(true, false).await?;
    nvim.set_current_buf(&buf).await?;
    nvim.command(format!("edit {}", file.display()).as_str())
        .await?;
    nvim.set_current_buf(&prev_buf).await?;
    Ok(buf)
}

async fn find_text_win(
    nvim: &Neovim<Compat<WriteHalf<UnixStream>>>,
) -> Result<Window<Compat<WriteHalf<UnixStream>>>> {
    for win in nvim.list_wins().await? {
        // If the windows current buffer is a normal buffer (editable)
        if win.get_buf().await?.get_option("buftype").await? == Value::String("".into()) {
            return Ok(win);
        }
    }
    Err(Error::NoWindow)
}
