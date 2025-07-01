use ratatui::style::{Color};
use tui_realm_stdlib::List;
use tuirealm::MockComponent;
use tuirealm::props::{Borders, TableBuilder, TextSpan, TextModifiers, Alignment};


use super::super::tui::MenuItem;

#[derive(Default)]
pub struct Menu {
    component: List,
    items: Vec<MenuItem>,
    selected: usize,
}

impl Menu {

    pub fn new(items: Vec<MenuItem>) -> Self {
        let mut component = List::default()
            .foreground(Color::Red)
            .background(Color::Blue)
            .highlighted_color(Color::Yellow)
            .highlighted_str("🚀")
            .modifiers(TextModifiers::BOLD)
            .scroll(true)
            .step(4)
            .borders(Borders::default())
            .title("events", Alignment::Center)
            .rewind(true)
            .rows(
                TableBuilder::default()
                    .add_col(TextSpan::from("Search"))
                    .add_col(TextSpan::from("Feed"))
                    .add_col(TextSpan::from("Exit"))
                    .add_row()
                    .build(),
            );

        Self {
            component,
            items,
            selected: 0,
        }
    }

    fn select(&mut self, index: usize) {
        self.selected = index;
        self.component = self.component.selected_line(index);
    }
}
