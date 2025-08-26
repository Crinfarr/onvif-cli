use ratatui::{crossterm::event::KeyEvent, prelude::CrosstermBackend, DefaultTerminal, Frame, Terminal};


pub trait RenderableScreen {
    fn handle_input(&mut self, input:KeyEvent) -> ();
    fn render(&self, frame:&mut Frame) -> ();
}