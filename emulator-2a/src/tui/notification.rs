use crate::helpers::{RED_BOLD, YELLOW, YELLOW_BOLD};
use tui::{buffer::Buffer, layout::Rect, widgets::StatefulWidget};

pub struct NotificationState {
    pub current: Option<String>,
}

impl NotificationState {
    /// Create a simple state without text.
    pub fn empty() -> Self {
        NotificationState { current: None }
    }
    /// Does the state contain any text?
    pub fn is_empty(&self) -> bool {
        self.current.is_none()
    }
    /// Drop the contained notification.
    pub fn clear(&mut self) {
        self.current = None;
    }
}

pub struct NotificationWidget;

impl StatefulWidget for NotificationWidget {
    type State = NotificationState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if let Some(ref text) = state.current {
            text.lines()
                .take(area.height.saturating_sub(1) as usize)
                .enumerate()
                .for_each(|(idx, line)| {
                    buf.set_stringn(
                        area.x,
                        area.y + idx as u16,
                        line,
                        area.width as usize,
                        *YELLOW,
                    );
                });
            if area.width > 9 {
                buf.set_string(area.width - 9, area.height, "<Any Key>", *YELLOW_BOLD);
            }
        } else {
            buf.set_stringn(
                area.x,
                area.y,
                "BUG: The notification widget is empty",
                area.width as usize,
                *RED_BOLD,
            );
        }
    }
}
