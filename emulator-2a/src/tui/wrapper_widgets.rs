use tui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{StatefulWidget, Widget},
};

use crate::helpers::LIGHTRED;

pub struct MinimumSize<W> {
    pub minimum_area: (u16, u16),
    pub inner: W,
}

impl<W> MinimumSize<W> {
    fn get_hint(&self, area: Rect) -> Option<&'static str> {
        let width_too_small = area.width < self.minimum_area.0;
        let height_too_small = area.height < self.minimum_area.1;
        if width_too_small && height_too_small {
            Some("Display width and height too small!")
        } else if width_too_small {
            Some("Display width too small!")
        } else if height_too_small {
            Some("Display height too small!")
        } else {
            None
        }
    }
}

impl<W: Widget> Widget for MinimumSize<W> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Render the hint or the actual widget if everything is alright
        if let Some(hint) = self.get_hint(area) {
            buf.set_stringn(area.x, area.y, hint, area.width as usize, *LIGHTRED);
        } else {
            self.inner.render(area, buf)
        }
    }
}

impl<W: StatefulWidget> StatefulWidget for MinimumSize<W> {
    type State = W::State;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Render the hint or the actual widget if everything is alright
        if let Some(hint) = self.get_hint(area) {
            buf.set_stringn(area.x, area.y, hint, area.width as usize, *LIGHTRED);
        } else {
            self.inner.render(area, buf, state)
        }
    }
}
