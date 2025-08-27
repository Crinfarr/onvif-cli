mod device_docs;
mod renderable_screen;
mod renderable_widget;
mod onvif;
mod screens {
    pub mod main_screen;
    pub mod confirm_exit;
}

use std::{str::FromStr, time::Duration};

use crossterm::event::KeyEvent;
use once_cell::sync::Lazy;

use color_eyre::Result;
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    style::Stylize,
    text::Line,
    widgets::{Block, Paragraph, Wrap},
};
use regex::Regex;

use crate::{device_docs::DeviceDoc, renderable_screen::RenderableScreen};

static IP_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^((2(([0-4]\d)|(5[0-5]))|([01]?\d?\d))\.){3}(2(([0-4]\d)|(5[0-5]))|([01]?\d?\d))$")
        .unwrap()
});

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let mut term = ratatui::init();
    term.clear()?;
    let app = App::default().main_loop(&mut term).await;
    ratatui::restore();
    app
}

#[derive(Debug, Default)]
pub struct App<'a> {
    screen: ScreenState,
    main:crate::screens::main_screen::MainScreen<'a>
}

#[derive(Debug, Default, Clone)]
enum ScreenState {
    #[default]
    MainScreen,
    HelpScreen(Box<ScreenState>),
}
enum BarStatus {
    Complete,
    Warning,
    Error,
}

impl App<'_> {
    pub async fn main_loop(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.main.exit {
            terminal.draw(|frame| {
                match self.screen {
                    ScreenState::HelpScreen(_) => unimplemented!(),
                    ScreenState::MainScreen => {
                        self.main.render(frame);
                    }
                }
            })?;
            if let Ok(has_event) = event::poll(Duration::from_secs(0)) {
                if has_event {
                    if let Event::Key(event) = event::read()? {
                        match self.screen {
                            ScreenState::MainScreen => self.main.handle_input(event),
                            ScreenState::HelpScreen(_) => unimplemented!(),
                        }
                    }
                }
            }
        }
        terminal.clear()?;
        Ok(())
    }
    /*fn draw(&self, frame: &mut Frame) {
        match &self.screen {
            ScreenState::MainScreen => {}
            ScreenState::HelpScreen(b_help_for) => {
                let help_for = b_help_for.as_ref();
                match help_for {
                    ScreenState::MainScreen => {
                        let help_win = Paragraph::new(include_str!("./main_help.txt"))
                            .block(Block::bordered().title("Help (1)").title_bottom(
                                Line::from("Press any key to exit".green()).right_aligned(),
                            ))
                            .wrap(Wrap { trim: false });
                        frame.render_widget(help_win, frame.area());
                    }
                    _ => {}
                }
            }
        }
    }*/
}
