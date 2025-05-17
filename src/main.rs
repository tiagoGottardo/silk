use std::error::Error;

use clap::{Parser, Subcommand};

mod terminal;
mod types;
mod ui;
mod youtube;

#[derive(Parser)]
#[command(name = "netimp")]
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
    let cli = Cli::parse();

    let mut terminal = terminal::init()?;

    match cli.command {
        Some(Commands::Open { url }) => {
            youtube::play_video::play_video(&mut terminal, &url).await?;
        }
        None => {
            ui::main_menu::menu_interface(&mut terminal).await?;
        }
    }

    terminal::exit(&mut terminal)?;
    Ok(())
}
