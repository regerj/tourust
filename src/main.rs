use app::App;
use clap::Parser;
use cli::Cli;
use error::Result;
use flexi_logger::FileSpec;

mod app;
mod cli;
mod error;
mod nvim;
mod tui;

#[tokio::main]
async fn main() -> Result<()> {
    //let _logger_handle = flexi_logger::Logger::try_with_str("debug")?
    //    .log_to_file(FileSpec::default())
    //    .start()?;
    let cli = Cli::parse();

    // create app and run it
    let mut app = App::new()?;
    if let Some(cmd) = cli.command {
        match cmd {
            cli::Command::Nvim(args) => {
                app.select_callback = Some(Box::new(move |x| {
                    nvim::select_callback(args.socket.clone(), x)
                }));
            }
        }
    }

    match app.run().await {
        Ok(_) => Ok(()),
        Err(err) => {
            log::error!("Error encountered: {}", err.to_string());
            Err(err)
        }
    }
}
