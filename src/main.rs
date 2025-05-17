use std::error::Error;

mod terminal;
mod types;
mod ui;
mod youtube;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut terminal = terminal::init()?;

    let _ = ui::main_menu::menu_interface(&mut terminal).await;

    terminal::exit(&mut terminal)?;
    Ok(())
}
