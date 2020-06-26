use ::tui::style::Color;
use ::tui::style::Modifier;
use ::tui::style::Style;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref DIMMED: Style = Style::default().modifier(Modifier::DIM);
    pub static ref YELLOW: Style = Style::default().fg(Color::Yellow);
    pub static ref RED: Style = Style::default().fg(Color::Red);
    pub static ref LIGHTRED: Style = Style::default().fg(Color::LightRed);
    pub static ref GREEN: Style = Style::default().fg(Color::Green);
}
