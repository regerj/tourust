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
    let handler = NvimHandler {};
    debug!("selection: {:?}", selection);

    // Get our API
    let (nvim, _) = nvim_rs::create::tokio::new_path(socket, handler)
        .await
        .unwrap_or_else(|err| {
            println!("Error occured: {}", err);
            panic!()
        });

    // Hide the current window
    //nvim.get_current_win().await?.hide().await?;

    // Select the proper window
    let wins = nvim.list_wins().await?;

    //for win in &wins {
    //    let buf = nvim.create_buf(false, true).await?;
    //    buf.set_lines(0, 0, false, vec!["Hello world :)".into()])
    //        .await?;
    //    let width = 20;
    //    let height = 3;
    //    let center_x = win.get_width().await? / 2;
    //    let center_y = win.get_height().await? / 2;
    //
    //    let mut config = Vec::new();
    //    config.push((
    //        Value::String("relative".into()),
    //        Value::String("win".into()),
    //    ));
    //    config.push((Value::String("win".into()), win.get_value().clone()));
    //    config.push((Value::String("width".into()), Value::Integer(width.into())));
    //    config.push((
    //        Value::String("height".into()),
    //        Value::Integer(height.into()),
    //    ));
    //    config.push((
    //        Value::String("col".into()),
    //        Value::Integer((center_x - width / 2).into()),
    //    ));
    //    config.push((
    //        Value::String("row".into()),
    //        Value::Integer((center_y - height / 2).into()),
    //    ));
    //    config.push((Value::String("focusable".into()), Value::Boolean(false)));
    //    config.push((
    //        Value::String("style".into()),
    //        Value::String("minimal".into()),
    //    ));
    //    let border = vec![
    //        Value::String("╭".into()),
    //        Value::String("─".into()),
    //        Value::String("╮".into()),
    //        Value::String("│".into()),
    //        Value::String("╯".into()),
    //        Value::String("─".into()),
    //        Value::String("╰".into()),
    //        Value::String("│".into()),
    //    ];
    //    config.push((Value::String("border".into()), Value::Array(border)));
    //
    //    nvim.open_win(&buf, false, config).await?;
    //}

    // Check if the correct file is already open in a buffer and switch to it
    //let buf = find_or_open_buf(&nvim, &selection).await?;
    //wins[0].set_buf(&buf).await?;
    //wins[0].set_cursor((selection.line as i64, selection.column as i64)).await?;


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
            wins[0].set_buf(&buf).await?;
            break;
        }
    }

    if let Err(err) = wins[0]
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
