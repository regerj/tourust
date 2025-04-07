use std::path::PathBuf;

use futures::StreamExt;
use log::{debug, error};
use nvim_rs::{compat::tokio::Compat, Buffer, Neovim, Value};
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
