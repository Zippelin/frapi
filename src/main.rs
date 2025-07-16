use eframe::App;
use eframe::Frame;
use egui::Context;

use crate::{settings::Settings, states::States, ui::UI};

pub mod executor;
pub mod settings;
pub mod states;
pub mod ui;

pub struct Frapi {
    ui: UI,
}

impl Frapi {
    pub fn new() -> Self {
        let settings = Settings::load();
        let states = States::from(&settings);
        let ui = UI::new(states);
        Self { ui }
    }

    pub fn run() {
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([800.0, 800.0])
                .with_min_inner_size([800.0, 600.]),
            ..Default::default()
        };

        let _ = eframe::run_native("FrAPI", options, Box::new(|_| Ok(Box::new(Self::new()))));
    }
}

impl App for Frapi {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        self.ui.update(ctx, frame);
    }
}

#[tokio::main]
async fn main() {
    Frapi::run();
}
