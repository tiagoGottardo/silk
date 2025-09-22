//! ## Model
//!
//! app model

use std::time::Duration;

use tokio::sync::mpsc;
use tuirealm::event::NoUserEvent;
use tuirealm::props::{Borders, Color};
use tuirealm::ratatui::layout::{Constraint, Direction, Layout};
use tuirealm::terminal::{CrosstermTerminalAdapter, TerminalAdapter, TerminalBridge};
use tuirealm::{Application, EventListenerCfg, Update};

use crate::types::ContentItem;
use crate::youtube::search_content;

use super::super::components::{Input, Menu};
use super::super::tui::{Id, Msg};

pub enum ActiveView {
    SearchResult,
    MainMenu,
    Idle,
}

pub struct Model<T>
where
    T: TerminalAdapter,
{
    pub app: Application<Id, Msg, NoUserEvent>,
    pub quit: bool,
    pub redraw: bool,
    pub terminal: TerminalBridge<T>,
    pub search_result: Vec<ContentItem>,
    pub active_view: ActiveView,
    pub tx: mpsc::Sender<Msg>,
}

impl Default for Model<CrosstermTerminalAdapter> {
    fn default() -> Self {
        let (tx, _rx) = mpsc::channel(1024);
        Self {
            app: Self::init_app(),
            quit: false,
            redraw: true,
            terminal: TerminalBridge::init_crossterm().expect("Cannot initialize terminal"),
            search_result: Vec::default(),
            active_view: ActiveView::MainMenu,
            tx,
        }
    }
}

impl<T> Model<T>
where
    T: TerminalAdapter,
{
    pub fn view(&mut self) {
        assert!(
            self.terminal
                .draw(|f| {
                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .margin(1)
                        .constraints([Constraint::Length(3), Constraint::Min(1)])
                        .split(f.area());

                    self.app.view(&Id::Input, f, chunks[0]);
                    self.app.view(&Id::Menu, f, chunks[1]);
                })
                .is_ok()
        );
    }

    fn init_app() -> Application<Id, Msg, NoUserEvent> {
        let mut app: Application<Id, Msg, NoUserEvent> = Application::init(
            EventListenerCfg::default()
                .crossterm_input_listener(Duration::from_millis(20), 3)
                .poll_timeout(Duration::from_millis(10))
                .tick_interval(Duration::from_secs(1)),
        );

        assert!(
            app.mount(
                Id::Input,
                Box::new(
                    Input::default()
                        .borders(Borders::default())
                        .foreground(Color::Green)
                        .label("Search on Youtube")
                ),
                Vec::default()
            )
            .is_ok()
        );

        assert!(
            app.mount(
                Id::Menu,
                Box::new(Menu::new(vec![
                    "Search".to_string(),
                    "Feed".to_string(),
                    "Exit".to_string()
                ])),
                Vec::default()
            )
            .is_ok()
        );

        assert!(app.active(&Id::Menu).is_ok());

        app
    }

    fn go_to_main_menu(&mut self) {
        assert!(
            self.app
                .remount(
                    Id::Menu,
                    Box::new(Menu::new(vec![
                        "Search".to_string(),
                        "Feed".to_string(),
                        "Exit".to_string()
                    ])),
                    Vec::default()
                )
                .is_ok()
        );
        self.active_view = ActiveView::MainMenu;
        assert!(self.app.active(&Id::Menu).is_ok());
    }
}

impl<T> Update<Msg> for Model<T>
where
    T: TerminalAdapter,
{
    fn update(&mut self, msg: Option<Msg>) -> Option<Msg> {
        if let Some(msg) = msg {
            self.redraw = true;

            match msg {
                Msg::AppClose => {
                    self.quit = true;
                }
                Msg::MenuSelected(item, idx) => match self.active_view {
                    ActiveView::MainMenu => match item.as_str() {
                        "Exit" => return Some(Msg::AppClose),
                        "Search" => {
                            assert!(self.app.active(&Id::Input).is_ok());
                            self.active_view = ActiveView::Idle;
                        }
                        _ => {}
                    },
                    ActiveView::SearchResult => {
                        let mut content_item = self.search_result[idx].clone();
                        tokio::spawn(async move {
                            content_item.play().await;
                        });
                    }
                    ActiveView::Idle => {}
                },

                Msg::Subscribe(_, idx) => {
                    let mut content_item = self.search_result[idx].clone();
                    tokio::spawn(async move {
                        content_item.subscribe().await;
                    });
                }
                Msg::Unsubscribe(_, idx) => {
                    let mut content_item = self.search_result[idx].clone();
                    tokio::spawn(async move {
                        content_item.unsubscribe().await;
                    });
                }
                Msg::Download(_, idx, video_track) => {
                    let mut content_item = self.search_result[idx].clone();
                    tokio::spawn(async move {
                        content_item.download(video_track).await;
                    });
                }
                Msg::Search(input) => {
                    let tx = self.tx.clone();
                    tokio::spawn(async move {
                        if let Ok(content) = search_content(&input).await {
                            tx.send(Msg::SearchResults(content)).await.ok();
                        }
                    });
                    self.active_view = ActiveView::Idle;
                    assert!(self.app.active(&Id::Menu).is_ok());
                }
                Msg::SearchResults(content) => {
                    self.search_result = content.clone();
                    let menu_items = content
                        .iter()
                        .map(|content_item| match content_item {
                            ContentItem::Video(video) => video.title.clone(),
                            ContentItem::Channel(channel) => channel.username.clone(),
                            ContentItem::Playlist(playlist) => playlist.title.clone(),
                        })
                        .collect();
                    self.active_view = ActiveView::SearchResult;
                    assert!(
                        self.app
                            .remount(Id::Menu, Box::new(Menu::new(menu_items)), Vec::default())
                            .is_ok()
                    );
                }
                Msg::Exit => match self.active_view {
                    ActiveView::MainMenu => return Some(Msg::AppClose),
                    ActiveView::SearchResult | ActiveView::Idle => self.go_to_main_menu(),
                },
                _ => {}
            }
        }
        None
    }
}
