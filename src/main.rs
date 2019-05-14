use pretty_env_logger;

use std::io;

pub mod schematic;
pub mod tui;

fn main() -> Result<(), io::Error> {
    pretty_env_logger::init();

    tui::run()
}
