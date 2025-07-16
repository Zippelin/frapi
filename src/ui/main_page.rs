use egui::Context;

use crate::{
    states::States,
    ui::main_page::{central_panel::CentralPanel, left_panel::LeftPanel},
};

mod central_panel;
mod left_panel;

pub struct MainPage {
    left_panel: LeftPanel,
    central_panel: CentralPanel,
}

impl MainPage {
    pub fn new() -> Self {
        Self {
            left_panel: LeftPanel::new(),
            central_panel: CentralPanel::new(),
        }
    }

    pub fn update(&mut self, ctx: &Context, states: &mut States) {
        self.left_panel.update(ctx, states);
        self.central_panel.update(ctx, states);
    }
}
