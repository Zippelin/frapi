#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use eframe::App;
use eframe::Frame;
use egui::Context;

use crate::{settings::Settings, states::States, ui::UI};

pub mod executor;
pub mod settings;
pub mod states;
pub mod ui;

pub struct Frapi {
    pub ui: UI,
}

// TODO: add served side abilities
impl Frapi {
    pub fn new() -> Self {
        let settings = Settings::load();
        let states = States::from(&settings);
        let ui = UI::new(states);
        Self { ui }
    }

    pub fn run() {
        let app_states = Self::new();

        let mut viewport = egui::ViewportBuilder::default()
            .with_inner_size(app_states.ui.states.options.window_size)
            .with_min_inner_size([800.0, 600.]);

        if let Some(pos) = app_states.ui.states.options.window_position {
            viewport = viewport.with_position(pos)
        };

        let options = eframe::NativeOptions {
            viewport,
            ..Default::default()
        };

        let _ = eframe::run_native("FrAPI", options, Box::new(|_| Ok(Box::new(app_states))));
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
