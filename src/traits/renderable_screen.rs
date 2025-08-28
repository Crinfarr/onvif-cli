use ratatui::{crossterm::event::KeyEvent, Frame};


pub trait RenderableScreen {
    fn handle_input(&mut self, input:KeyEvent) -> ();
    fn render(&self, frame:&mut Frame) -> ();
}