mod device_docs;
mod renderable_screen;
mod renderable_widget;
mod onvif;
mod screens {
    mod main_screen;
    mod confirm_exit;
}

use std::{str::FromStr};

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

use crate::device_docs::DeviceDoc;

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
    warn_exit: bool,
    exit: bool,
    screen: ScreenState,
    ip_addrs: Vec<DeviceDoc>,
    ip_sel_idx: usize,
    need_rerender:bool
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
        while !self.exit {
            if self.need_rerender {
                terminal.draw(|frame| self.draw(frame))?;
            }
            if let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press
            {
                if self.warn_exit {
                    if key.code == KeyCode::Esc {
                        self.exit = true;
                    } else {
                        self.warn_exit = false;
                    }
                    continue;
                }
                match &self.screen {
                    ScreenState::MainScreen => match key.code {
                        KeyCode::Esc => {
                            if !self.warn_exit {
                                self.warn_exit = true
                            }
                        }
                        KeyCode::Enter => {
                            self.mv_prompt.delete_line_by_head();
                            let text = self.mv_prompt.yank_text();
                            let mut splt = text.trim().split(' ').filter(|s| !s.is_empty());
                            if let Some(cmd) = splt.next() {
                                match cmd {
                                    "add" => {
                                        if let Some(arg) = splt.next() {
                                            if let Some(ip) = IP_REGEX.find(arg) {
                                                self.set_prompt_status(
                                                    format!("Added {}", ip.as_str()),
                                                    BarStatus::Complete,
                                                );
                                                self.ip_addrs
                                                    .push(DeviceDoc::from_str(ip.into()).unwrap());
                                            } else {
                                                self.set_prompt_status(
                                                    format!("{} is not a valid IPv4", arg),
                                                    BarStatus::Error,
                                                );
                                            }
                                        } else {
                                            self.set_prompt_status(
                                                "No argument provided for Add operator".into(),
                                                BarStatus::Error,
                                            );
                                        }
                                    }
                                    "del" | "rem" => {
                                        if self.ip_addrs.len() == 0 {
                                            self.set_prompt_status(
                                                "No items to remove".into(),
                                                BarStatus::Warning,
                                            );
                                            continue;
                                        }
                                        if let Some(usr_input) = splt.next() {
                                            if let Ok(num) = i16::from_str_radix(usr_input, 10) {
                                                if num as usize >= self.ip_addrs.len() {
                                                    self.set_prompt_status(format!("Provided number {} is too high (max: {})", num, self.ip_addrs.len()-1), BarStatus::Warning);
                                                    continue;
                                                }
                                                let rmd = self.ip_addrs.remove(num as usize);
                                                if self.ip_sel_idx > 0//don't underflow while stepping up
                                                    && (self.ip_sel_idx == self.ip_addrs.len()//step up if at last position
                                                        || self.ip_sel_idx > num as usize)
                                                //    or if removing before cursor
                                                {
                                                    self.ip_sel_idx -= 1;
                                                }
                                                self.set_prompt_status(
                                                    format!("Removed {}", rmd.ip),
                                                    BarStatus::Complete,
                                                );
                                            } else {
                                                self.set_prompt_status(
                                                    format!("Expected a number, got {}", usr_input),
                                                    BarStatus::Error,
                                                );
                                            }
                                        } else {
                                            let rmd =
                                                self.ip_addrs.remove(self.ip_sel_idx as usize);
                                            if self.ip_sel_idx == self.ip_addrs.len()
                                                && self.ip_sel_idx > 0
                                            {
                                                self.ip_sel_idx -= 1;
                                            }
                                            self.set_prompt_status(
                                                format!("Removed selected ({})", rmd.ip),
                                                BarStatus::Complete,
                                            );
                                        }
                                    }
                                    "?" | "help" => {
                                        self.screen = ScreenState::HelpScreen(Box::new(
                                            ScreenState::MainScreen,
                                        ));
                                    }
                                    "exit" | "quit" | "close" => {
                                        self.set_prompt_status(
                                            "Press esc to exit".into(),
                                            BarStatus::Warning,
                                        );
                                    }
                                    unknown => {
                                        self.set_prompt_status(
                                            format!("Unknown command {}", unknown),
                                            BarStatus::Warning,
                                        );
                                    }
                                }
                            }
                        }
                        KeyCode::Up => {
                            if self.ip_sel_idx > 0 {
                                self.ip_sel_idx -= 1;
                            }
                        }
                        KeyCode::Down => {
                            if self.ip_sel_idx + 1 < self.ip_addrs.len() {
                                self.ip_sel_idx += 1;
                            }
                        }
                        _keycode => if self.mv_prompt.input(key) {},
                    },
                    ScreenState::HelpScreen(old) => match key.code {
                        _ => {
                            self.screen = old.as_ref().clone();
                        }
                    },
                }
            }
        }
        terminal.clear()?;
        Ok(())
    }

    fn set_prompt_status(&mut self, message: String, status: BarStatus) {
        self.mv_prompt.set_block(
            Block::bordered()
                .border_style(Style::default().fg(match status {
                    BarStatus::Complete => Color::Blue,
                    BarStatus::Warning => Color::LightYellow,
                    BarStatus::Error => Color::Red,
                }))
                .title(message),
        );
    }

    fn draw(&self, frame: &mut Frame) {
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
    }
}
