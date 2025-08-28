mod device_docs;
mod traits;
mod onvif;
mod screens {
    pub mod confirm_exit;
    pub mod main_screen;
}

use std::time::Duration;

use once_cell::sync::Lazy;

use color_eyre::Result;
use ratatui::{
    DefaultTerminal,
    crossterm::event::{self, Event},
};
use regex::Regex;

use crate::traits::renderable_screen::RenderableScreen;

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
    main: crate::screens::main_screen::MainScreen<'a>,
}

#[derive(Debug, Default, Clone)]
enum ScreenState {
    #[default]
    MainScreen,
    HelpScreen(Box<ScreenState>),
}

impl App<'_> {
    pub async fn main_loop(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.main.exit {
            terminal.draw(|frame| match self.screen {
                ScreenState::HelpScreen(_) => unimplemented!(),
                ScreenState::MainScreen => {
                    self.main.render(frame);
                }
            })?;
            if let Ok(has_event) = event::poll(Duration::from_secs(0))
                && has_event
                    && let Event::Key(event) = event::read()? {
                        match self.screen {
                            ScreenState::MainScreen => self.main.handle_input(event),
                            ScreenState::HelpScreen(_) => unimplemented!(),
                        }
                    }
        }
        terminal.clear()?;
        Ok(())
    }
}
