mod config;
mod crypto;
mod geo;
mod i18n;
mod ui;

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    ui::run()
}
