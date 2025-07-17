//! ## Components
//!
//! demo example components

use tuirealm::props::{Alignment, Borders, Color, Style};
use tuirealm::ratatui::widgets::Block;

pub mod input;
pub mod label;
pub mod menu;

pub use input::Input;
pub use label::Label;
pub use menu::Menu;

/// ### `get_block`
///
/// Get block
pub(crate) fn get_block<'a>(props: Borders, title: (String, Alignment), focus: bool) -> Block<'a> {
    Block::default()
        .borders(props.sides)
        .border_style(if focus {
            props.style()
        } else {
            Style::default().fg(Color::Reset).bg(Color::Reset)
        })
        .border_type(props.modifiers)
        .title(title.0)
        .title_alignment(title.1)
}
