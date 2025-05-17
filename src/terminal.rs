use std::{
    io::{self, Stdout},
    process::{self},
};

type Terminal = ratatui::Terminal<CrosstermBackend<Stdout>>;
use ratatui::{
    crossterm::{
        execute,
        terminal::{LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    },
    prelude::CrosstermBackend,
};

pub fn init() -> Result<Terminal, io::Error> {
    enable_raw_mode()?;
    let stdout = std::io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;

    Ok(terminal)
}

pub fn exit(terminal: &mut Terminal) -> Result<(), io::Error> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    process::exit(0);
}
