use eframe::Frame;
use egui::{Context, Key, ThemePreference};

use crate::{
    settings::Settings,
    states::States,
    ui::{bottom::BottomPanel, main_menu::MainMenu, main_page::MainPage},
};

mod bottom;
pub mod colors;
pub mod fonts;
mod icons;
mod main_menu;
mod main_page;

/// UI states of data and window
/// Add to `fn controls()` - to add mode keybinding shortcuts
pub struct UI {
    /// States pf application
    pub states: States,
    /// Main Page repr for draw
    main_page: MainPage,
    /// Main Menu repr for draw
    main_menu: MainMenu,
    /// Bottom Page repr for draw
    bottom: BottomPanel,
    /// Frames before global save occures
    /// 0 - max count, 1 - current counter
    throttling: (usize, usize),
}

impl UI {
    pub fn new(states: States) -> Self {
        Self {
            states,
            main_page: MainPage::new(),
            main_menu: MainMenu::new(),
            bottom: BottomPanel::new(),
            throttling: (240, 0),
        }
    }

    pub fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        let _ = frame;

        if self.states.style.is_dark_theme() {
            ctx.set_theme(ThemePreference::Dark);
        } else {
            ctx.set_theme(ThemePreference::Light);
        }

        self.main_menu.update(ctx, &mut self.states);
        self.bottom.update(ctx, &mut self.states);
        match self.states.current_page {
            crate::states::Page::MAIN => self.main_page.update(ctx, &mut self.states),
        }

        // Checking if move is possible.
        // We must check it after all possible iterations over antities done.
        self.states.update();
        self.controls(ctx);
        self.slow_autoupdate(ctx);
    }

    /// Try slow action to perform. Once per `throttling.0` frames
    fn slow_autoupdate(&mut self, ctx: &Context) {
        if self.throttling.0 == self.throttling.1 {
            self.throttling.1 = 0;

            let mut is_changed = false;

            let current_size = ctx.used_size();
            if current_size != self.states.options.window_size {
                self.states.options.window_size = current_size;
                is_changed = true;
            }

            let current_position = ctx.viewport(|r| match r.input.raw.viewport().outer_rect {
                Some(pos) => Some(pos.min),
                None => None,
            });

            if current_position.is_some() {
                let current = current_position.unwrap();

                if self.states.options.window_position.is_none() {
                    self.states.options.window_position = Some(current);
                    is_changed = true;
                } else {
                    let old_position = self.states.options.window_position.unwrap();
                    if current != old_position {
                        self.states.options.window_position = Some(current);
                        is_changed = true;
                    }
                }
            }

            if is_changed {
                let settings = Settings::from_original(&self.states);
                let _ = settings.save(None);
            }
        } else {
            self.throttling.1 += 1;
        }
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
