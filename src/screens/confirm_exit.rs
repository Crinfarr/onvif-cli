use ratatui::{
    crossterm::event::{KeyCode, KeyEvent}, layout::{Constraint, Flex, Layout}, style::{Color, Style, Stylize}, text::{Line, Span}, widgets::{Block, Clear, Paragraph}, Frame
};

use crate::traits::renderable_screen::RenderableScreen;

#[derive(Debug, Default)]
pub struct ConfirmExitScreen {
    pub should_exit:Option<bool>
}
impl ConfirmExitScreen {}
impl RenderableScreen for ConfirmExitScreen {
    fn handle_input(&mut self, input: KeyEvent) {
        match input.code {
            KeyCode::Esc => {
                self.should_exit = Some(true)
            }
            _ => {
                self.should_exit = Some(false)
            }
        }
    }

    fn render(&self, frame: &mut Frame) {
        let outerbox = Block::bordered()
            .title("Exit warning")
            .border_style(Style::new().fg(Color::Red))
            .bg(Color::Black);
        let warnbox = Paragraph::new(Line::from(vec![
            Span::raw("Press"),
            " esc ".fg(Color::Red),
            Span::raw("again to exit, any other key to go back"),
        ]))
        .alignment(ratatui::layout::Alignment::Center);
        let [area] = Layout::horizontal([Constraint::Fill(100)])
            .flex(Flex::Center)
            .areas(frame.area());
        let [area] = Layout::vertical([Constraint::Percentage(20)])
            .flex(Flex::Center)
            .areas(area);
        let [textline] = Layout::vertical([Constraint::Length(1)])
            .flex(Flex::Center)
            .areas(area);
        frame.render_widget(Clear, area);
        frame.render_widget(outerbox, area);
        frame.render_widget(warnbox, textline);
    }
}
