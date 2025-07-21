use eframe::Frame;
use egui::{Context, Key};

use crate::{
    states::States,
    ui::{bottom::BottomPanel, main_menu::MainMenu, main_page::MainPage},
};

mod bottom;
pub mod colors;
pub mod fonts;
mod icons;
mod main_menu;
mod main_page;

pub struct UI {
    states: States,
    main_page: MainPage,
    main_menu: MainMenu,
    bottom: BottomPanel,
}

impl UI {
    pub fn new(states: States) -> Self {
        Self {
            states,
            main_page: MainPage::new(),
            main_menu: MainMenu::new(),
            bottom: BottomPanel::new(),
        }
    }

    pub fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        let _ = frame;
        self.main_menu.update(ctx, &mut self.states);
        self.bottom.update(ctx, &mut self.states);
        match self.states.current_page {
            crate::states::Page::MAIN => self.main_page.update(ctx, &mut self.states),
        }

        // Checking if move is possible.
        // We must check it after all possible iterations over antities done.
        self.states.update();
        self.controls(ctx);
    }

    /// Process all key shortcats
    fn controls(&mut self, ctx: &Context) {
        // Save selectted item changes
        if ctx.input(|input| {
            input.key_pressed(Key::S) && (input.modifiers.ctrl || input.modifiers.command)
        }) {
            self.states.save_selected(None);
        }

        // Save all changes
        if ctx.input(|input| {
            input.key_pressed(Key::S)
                && (input.modifiers.shift && (input.modifiers.ctrl || input.modifiers.command))
        }) {
            self.states.save(None);
        }
    }
}
