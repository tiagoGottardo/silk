use std::error::Error;

use clap::{Parser, Subcommand};
use silk::{
    config::{db, env},
    terminal, tui,
    youtube::{self, update_feed},
};

fn setup_logger() -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {}] {}",
                record.level(),
                record.target(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(fern::log_file("debug.log")?)
        .apply()?;
    Ok(())
}

#[derive(Parser)]
#[command(name = "silk")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Open and play a video directly from a URL
    Open {
        /// The URL of the video to play
        url: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    setup_logger()?;

    env::Env::init();
    db::init().await;

    update_feed().await;

    let cli = Cli::parse();

    let mut terminal = terminal::init()?;

    match cli.command {
        Some(Commands::Open { url }) => {
            youtube::play_video(&mut terminal, &url).await?;
        }
        None => {
            tui::tui::main();
        }
    }

    terminal::exit(&mut terminal)?;
    Ok(())
}
