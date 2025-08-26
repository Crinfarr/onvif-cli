use once_cell::sync::Lazy;
use ratatui::{
    Frame,
    crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers},
    layout::{Constraint, Flex, Layout, Rect},
    style::{Color, Style, Stylize},
    symbols,
    text::Span,
    widgets::{Block, Borders, List, Paragraph, Wrap},
};
use regex::Regex;
use tui_textarea::TextArea;

use crate::{
    device_docs::DeviceDoc, renderable_screen::RenderableScreen,
    renderable_widget::RenderableWidget,
};

#[derive(Debug, Default)]
pub struct MainScreen<'a> {
    ip_addrs: Vec<DeviceDoc>,
    inputbox: PromptBox<'a>,
    iplist: IpList<'a>,
    try_exit: bool,
}
impl MainScreen<'_> {}
impl RenderableScreen for MainScreen<'_> {
    fn handle_input(&mut self, input: ratatui::crossterm::event::KeyEvent) -> () {
        if input.kind != KeyEventKind::Press {
            return; //Only respond once in terminals that count press and release as two events
        }
        match input.code {
            KeyCode::Esc => {
                self.try_exit = true;
            }
            _ => self.inputbox.handle_input(input),
        }
    }
    fn render(&self, frame: &mut Frame) -> () {
        let [top_half, prompt] =
            Layout::vertical([Constraint::Percentage(90), Constraint::Length(3)])
                .flex(Flex::Center)
                .areas(frame.area());
        let [list_area, detail] = Layout::horizontal([Constraint::Fill(1), Constraint::Fill(1)])
            .flex(Flex::Start)
            .areas(top_half);
        let iplist = List::default()
            .block(
                Block::bordered()
                    .borders(Borders::ALL)
                    .border_set(symbols::border::Set {
                        top_right: symbols::line::NORMAL.horizontal_down,
                        bottom_right: symbols::line::NORMAL.horizontal_up,
                        ..Default::default()
                    }),
            )
            .items(
                self.ip_addrs
                    .iter()
                    .enumerate()
                    .map(|(index, dev_doc)| {
                        if index == self.ip_sel_idx.into() {
                            format!("{}. {}", index, dev_doc.ip).black().on_white()
                        } else {
                            format!("{}. {}", index, dev_doc.ip).into()
                        }
                    })
                    .collect::<Vec<Span>>(),
            );
        let details = Paragraph::new(
            "there will be a widget here with formatted details. eventually. dont count on it",
        )
        .block(Block::bordered().borders(Borders::ALL ^ Borders::LEFT))
        .wrap(Wrap::default());
        self.iplist.render(frame, list_area);
        frame.render_widget(details, detail);
        self.inputbox.render(frame, prompt);
    }
}

//IP list
#[derive(Debug, Default)]
struct IpList<'a> {
    inner: List<'a>,
    selected: usize,
    ips: Vec<DeviceDoc>,
}
impl IpList<'_> {}
impl RenderableWidget for IpList<'_> {
    fn handle_input(&mut self, key: KeyEvent) -> () {
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
    fn render(&self, frame: &mut Frame, area: Rect) -> () {
        frame.render_widget(&self.inner, area);
    }
}

//promptbox
static IP_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^((2(([0-4]\d)|(5[0-5]))|([01]?\d?\d))\.){3}(2(([0-4]\d)|(5[0-5]))|([01]?\d?\d))$")
        .unwrap()
});
enum BarStatus {
    Complete,
    Warning,
    Error,
}
enum CommandSignal {
    AddItem(String),
    DelItem(Option<usize>),
    Help,
    Exit,
}
#[derive(Debug)]
struct PromptBox<'a> {
    inner: TextArea<'a>,
    handler: fn(CommandSignal) -> (),
}
impl Default for PromptBox<'_> {
    fn default() -> Self {
        let mut ta = TextArea::default();
        ta.set_block(Block::bordered());
        PromptBox {
            inner: ta,
            handler: |_| unimplemented!("No command handler specified"),
        }
    }
}
impl PromptBox<'_> {
    fn set_status(&mut self, message: String, status: BarStatus) {
        self.inner.set_block(
            Block::bordered()
                .border_style(Style::default().fg(match status {
                    BarStatus::Complete => Color::Blue,
                    BarStatus::Error => Color::Red,
                    BarStatus::Warning => Color::LightYellow,
                }))
                .title(message),
        );
    }
    fn set_command_handler(&mut self, handler: fn(CommandSignal) -> ()) {
        self.handler = handler;
    }
}
impl RenderableWidget for PromptBox<'_> {
    fn handle_input(&mut self, input: KeyEvent) -> () {
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
                                    (self.handler)(CommandSignal::AddItem(ip.as_str().to_string()));
                                }
                            }
                        }
                        "del" | "rm" => (self.handler)(CommandSignal::DelItem(
                            if let Some(uinput) = split.next() {
                                if let Ok(num) = usize::from_str_radix(uinput, 10) {
                                    Some(num)
                                } else {
                                    None
                                }
                            } else {
                                None
                            },
                        )),
                        "?"|"help" => (self.handler)(CommandSignal::Help),
                        "exit"|"quit"|"close" => (self.handler)(CommandSignal::Exit),
                        unknown => self.set_status(format!("Unknown command {}", unknown), BarStatus::Warning),
                    }
                }
            }
            _ => if self.inner.input(input) {},
        }
    }
    fn render(&self, frame: &mut Frame, area: Rect) -> () {
        frame.render_widget(&self.inner, area);
    }
}
