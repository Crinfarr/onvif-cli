use color_eyre::Result;
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode},
    layout::{Constraint, Flex, Layout},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, Paragraph, Wrap},
};
use tui_textarea::TextArea;

fn main() -> Result<()> {
    color_eyre::install()?;
    let mut term = ratatui::init();
    let app = App::default().main_loop(&mut term);
    ratatui::restore();
    app
}

#[derive(Debug, Default)]
pub struct App<'a> {
    warnExit: bool,
    exit: bool,
    screen: ScreenState,
    ipAddrs: Vec<String>,
    mv_prompt: TextArea<'a>,
}

#[derive(Debug, Default)]
enum ScreenState {
    #[default]
    MainScreen,
}

impl App<'_> {
    pub fn main_loop(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        self.mv_prompt.set_block(Block::bordered());
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            if let Event::Key(key) = event::read()? {
                if self.warnExit {
                    if key.code == KeyCode::Esc {
                        self.exit = true;
                    }
                    else {
                        self.warnExit = false;
                    }
                    continue;
                }
                match self.screen {
                    ScreenState::MainScreen => match key.code {
                        KeyCode::Esc => {
                            if !self.warnExit {
                                self.warnExit = true
                            }
                        }
                        KeyCode::Enter => {
                            //TODO input validation and processing
                            self.mv_prompt.delete_line_by_head();
                            let _text = self.mv_prompt.yank_text();
                        }
                        _keycode => {
                            if self.mv_prompt.input(key) {}
                        }
                    },
                }
            }
        }
        return Ok(());
    }

    fn draw(&self, frame: &mut Frame) {
        match self.screen {
            ScreenState::MainScreen => {
                let [top_half, prompt] = Layout::vertical([Constraint::Percentage(90), Constraint::Min(3)])
                    .flex(Flex::Center)
                    .areas(frame.area());
                let [list_area, popout] = Layout::horizontal([Constraint::Fill(1), Constraint::Fill(1)])
                    .flex(Flex::Start)
                    .areas(top_half);
                let iplist = List::default()
                    .block(Block::bordered())
                    .items(self.ipAddrs.clone());
                let details = Paragraph::new("there will be a widget here with formatted details. eventually. dont count on it")
                    .block(Block::bordered())
                    .wrap(Wrap::default());
                frame.render_widget(iplist, list_area);
                frame.render_widget(details, popout);
                frame.render_widget(&self.mv_prompt, prompt);
                //OVERLAYS DO THESE LAST
                if self.warnExit {
                    let warnbox = Paragraph::new(Line::from(vec![
                        Span::raw("Press"),
                        " esc ".fg(Color::Red),
                        Span::raw("to exit, any other key to go back"),
                    ]))
                    .alignment(ratatui::layout::Alignment::Center)
                    .block(
                        Block::bordered()
                            .title("Exit warning")
                            .border_style(Style::new().fg(Color::Red))
                            .bg(Color::Black),
                    );
                    let [area] = Layout::horizontal([Constraint::Fill(100)])
                        .flex(Flex::Center)
                        .areas(frame.area());
                    let [area] = Layout::vertical([Constraint::Percentage(20)])
                        .flex(Flex::Center)
                        .areas(area);
                    frame.render_widget(Clear, area);
                    frame.render_widget(warnbox, area);
                }
            }
        }
    }
}
