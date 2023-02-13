use std::collections::HashSet;

use tui::{buffer::Buffer, layout::Rect, style::Color, widgets::Widget};

pub struct DisplayCanvas<'a> {
    display: &'a HashSet<(i32, i32)>,
}

impl<'a> DisplayCanvas<'a> {
    pub fn new(display: &HashSet<(i32, i32)>) -> DisplayCanvas {
        DisplayCanvas { display }
    }
}

impl<'a> Widget for DisplayCanvas<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        for x in area.left()..area.right() {
            for y in area.top()..area.bottom() {
                let cell_on = self.display.contains(&(x as i32, y as i32));
                let color = if cell_on { Color::White } else { Color::Black };
                buf.get_mut(x, y).set_bg(color);
            }
        }
    }
}
