use std::path::PathBuf;

use log::{debug, error};
use nvim_rs::Value;

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

    if let Err(err) = nvim
        .get_current_win()
        .await?
        .set_cursor((selection.line as i64, selection.column as i64))
        .await
    {
        error!("Failed to set cursor: {}", err.to_string());
        panic!()
    }
    Ok(())
}
