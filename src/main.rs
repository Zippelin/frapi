use crate::{settings::Settings, states::Pages};

pub mod executor;
pub mod settings;
pub mod states;

pub struct Frapi {
    settings: Settings,
    pages: Pages,
}

impl Frapi {
    pub fn new() -> Self {
        let settings = Settings::load();
        let pages = Pages::from(&settings);
        Self { settings, pages }
    }
}

fn main() {
    println!("Hello, world!");
}
