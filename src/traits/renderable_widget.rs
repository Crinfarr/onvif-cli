use ratatui::{Frame, crossterm::event::KeyEvent, layout::Rect};

pub trait RenderableWidget {
    fn handle_input(&mut self, key: KeyEvent) -> ();
    fn render(&self, frame: &mut Frame, area: Rect) -> ();
}
