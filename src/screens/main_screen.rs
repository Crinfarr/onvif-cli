use once_cell::sync::Lazy;
use ratatui::{
    Frame,
    crossterm::event::{KeyCode, KeyEvent, KeyEventKind},
    layout::{Constraint, Flex, Layout, Rect},
    style::{Style, Stylize},
    symbols,
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};
use regex::Regex;
use tui_textarea::TextArea;

use crate::{
    device_docs::DeviceDoc,
    screens::confirm_exit::ConfirmExitScreen,
    traits::{RenderableScreen, RenderableWidget},
};

#[derive(Debug, Default)]
pub struct MainScreen<'a> {
    inputbox: PromptBox<'a>,
    iplist: IpList<'a>,
    exit_popup: ConfirmExitScreen,
    try_exit: bool,
    pub exit: bool,
}
impl MainScreen<'_> {}
impl RenderableScreen for MainScreen<'_> {
    fn handle_input(&mut self, input: ratatui::crossterm::event::KeyEvent) {
        if self.try_exit {
            match input {
                any_input => self.exit_popup.handle_input(any_input),
            }
            if let Some(should_exit) = self.exit_popup.should_exit {
                self.exit = should_exit;
                if !should_exit {
                    self.try_exit = false;
                }
                return;
            }
        }
        if input.kind != KeyEventKind::Press {
            return; //Only respond once in terminals that count press and release as two events
        }
        match input.code {
            KeyCode::Esc => {
                self.try_exit = true;
            }
            KeyCode::Up | KeyCode::Down => self.iplist.handle_input(input),
            _ => self.inputbox.handle_input(input),
        }
    }
    fn render(&self, frame: &mut Frame) {
        let [top_half, prompt] =
            Layout::vertical([Constraint::Percentage(90), Constraint::Length(3)])
                .flex(Flex::Center)
                .areas(frame.area());
        let [list_area, detail] = Layout::horizontal([Constraint::Fill(1), Constraint::Fill(1)])
            .flex(Flex::Start)
            .areas(top_half);
        let details = Paragraph::new(
            "there will be a widget here with formatted details. eventually. dont count on it",
        )
        .block(Block::bordered().borders(Borders::ALL ^ Borders::LEFT))
        .wrap(Wrap::default());
        self.iplist.render(frame, list_area);
        frame.render_widget(details, detail);
        self.inputbox.render(frame, prompt);
        //overlay
        if self.try_exit {
            self.exit_popup.render(frame);
        }
    }
}

//IP list
#[derive(Debug)]
struct IpList<'a> {
    inner: List<'a>,
    selected: usize,
    ips: Vec<DeviceDoc>,
}
impl IpList<'_> {}
impl RenderableWidget for IpList<'_> {
    fn handle_input(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up => {
                if self.selected > 0 {
                    self.selected -= 1
                }
            }
            KeyCode::Down => {
                if self.selected < self.ips.len() {
                    self.selected += 1
                }
            }
            code => {
                unimplemented!("IpList does not handle keycode {}", code)
            }
        }
    }
    fn render(&self, frame: &mut Frame, area: Rect) {
        let renderable =
            self.inner
                .clone()
                .items(self.ips.iter().enumerate().map(|(index, doc)| {
                    if index == self.selected {
                        ListItem::new(format!("{}> {}", index, doc.ip))
                    } else {
                        ListItem::new(format!("{}> {}", index, doc.ip))
                            .black()
                            .on_white()
                    }
                }));
        frame.render_widget(renderable, area);
    }
}
impl Default for IpList<'_> {
    fn default() -> Self {
        IpList {
            inner: List::default().block(Block::bordered().borders(Borders::ALL).border_set(
                symbols::border::Set {
                    top_right: symbols::line::NORMAL.horizontal_down,
                    bottom_right: symbols::line::NORMAL.horizontal_up,
                    ..Default::default()
                },
            )),
            ips: Vec::default(),
            selected: usize::default(),
        }
    }
}

//promptbox
static IP_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^((2(([0-4]\d)|(5[0-5]))|([01]?\d?\d))\.){3}(2(([0-4]\d)|(5[0-5]))|([01]?\d?\d))$")
        .unwrap()
});
enum BarStatus {
    Complete(String),
    Warning(String),
    Error(String),
}
#[derive(Debug)]
pub enum CommandSignal {
    AddItem(String),
    DelItem(Option<usize>),
    Help,
    Exit,
}
#[derive(Debug)]
struct PromptBox<'a> {
    inner: TextArea<'a>,
    command: Option<CommandSignal>,
}
impl Default for PromptBox<'_> {
    fn default() -> Self {
        let mut ta = TextArea::default();
        ta.set_block(Block::bordered());
        PromptBox {
            inner: ta,
            command: None,
        }
    }
}
impl PromptBox<'_> {
    fn set_status(&mut self, status: BarStatus) {
        match status {
            BarStatus::Complete(msg) => self.inner.set_block(
                Block::bordered()
                    .border_style(Style::default().blue())
                    .title(msg),
            ),
            BarStatus::Error(msg) => self.inner.set_block(
                Block::bordered()
                    .border_style(Style::default().red())
                    .title(msg),
            ),
            BarStatus::Warning(msg) => self.inner.set_block(
                Block::bordered()
                    .border_style(Style::default().light_yellow())
                    .title(msg),
            ),

            #[allow(unreachable_patterns)]
            _ => unimplemented!("No status handler"),
        }
    }
}
impl RenderableWidget for PromptBox<'_> {
    fn handle_input(&mut self, input: KeyEvent) {
        match input.code {
            KeyCode::Enter => {
                self.inner.delete_line_by_head();
                let text = self.inner.yank_text();
                let mut split = text.trim().split(' ').filter(|s| !s.is_empty());
                if let Some(cmd) = split.next() {
                    match cmd {
                        "add" => {
                            if let Some(arg) = split.next() {
                                if let Some(ip) = IP_REGEX.find(arg) {
                                    if self.command.is_some() {
                                        panic!("Active command was not consumed")
                                    }
                                    self.command =
                                        Some(CommandSignal::AddItem(ip.as_str().to_string()));
                                } else {
                                    self.set_status(BarStatus::Warning(format!(
                                        "{} is not a valid ipv4",
                                        arg
                                    )));
                                }
                            } else {
                                self.set_status(BarStatus::Error("Can't add nothing".to_string()));
                            }
                        }
                        "del" | "rm" => {
                            if let Some(uinput) = split.next() {
                                if let Ok(index) = str::parse::<usize>(uinput) {
                                    if self.command.is_some() {
                                        panic!("Active command was not consumed")
                                    }
                                    self.command = Some(CommandSignal::DelItem(Some(index)))
                                } else {
                                    self.set_status(BarStatus::Error(format!(
                                        "Cannot parse index {}",
                                        uinput
                                    )));
                                }
                            } else {
                                if self.command.is_some() {
                                    panic!("Active command was not consumed")
                                }
                                self.command = Some(CommandSignal::DelItem(None))
                            }
                        }
                        "?" | "help" => {
                            if self.command.is_some() {
                                panic!("Active command was not consumed")
                            }
                            self.command = Some(CommandSignal::Help);
                        }
                        "exit" | "quit" | "close" => {
                            if self.command.is_some() {
                                panic!("Active command was not consumed")
                            }
                            self.command = Some(CommandSignal::Exit);
                        }
                        unknown => self
                            .set_status(BarStatus::Warning(format!("Unknown command {}", unknown))),
                    }
                }
            }
            _ => if self.inner.input(input) {},
        }
    }
    fn render(&self, frame: &mut Frame, area: Rect) {
        frame.render_widget(&self.inner, area);
    }
}
