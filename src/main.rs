use clap::{Parser, Subcommand};
use silk::{
    config::{db, env},
    terminal, ui, youtube,
};
use std::error::Error;

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
    env::Env::init();
    db::init().await;

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
