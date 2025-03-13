use app::App;
use clap::Parser;
use cli::Cli;
use error::Result;

mod app;
mod error;
mod tui;
mod cli;
mod nvim;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // create app and run it
    let mut app = App::new()?;
    if let Some(cmd) = cli.command {
        match cmd {
            cli::Command::Nvim(args) => {
                app.select_callback = Some(Box::new(move |x| nvim::select_callback(args.socket.clone(), x)));
            }
        }
    }

    app.run().await?;

    Ok(())
}
