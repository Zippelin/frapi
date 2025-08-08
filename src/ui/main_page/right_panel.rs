use egui::{vec2, Button, Context, Frame, SidePanel};

use crate::{
    states::{main_page::RightPanelType, States},
    ui::{icons::Icon, main_page::right_panel::right_panel_events::RightPanelEvents},
};

pub mod right_panel_events;

pub struct RightPanel {
    events: RightPanelEvents,
}

impl RightPanel {
    pub fn new() -> Self {
        Self {
            events: RightPanelEvents::new(),
        }
    }
    pub fn update(&mut self, ctx: &Context, states: &mut States) {
        if states.main_page.right_panel.is_visible {
            match states.main_page.right_panel.panel_type {
                RightPanelType::EVENTS => self.events.update(ctx, states),
            }
        }

        SidePanel::right("right-panel-buttons")
            .default_width(30.)
            .resizable(false)
            .frame(Frame::new().fill(states.style.color_main()))
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.style_mut().spacing.button_padding = vec2(10., 10.);
                    if ui.add(Button::new(Icon::list())).clicked() {
                        states.main_page.right_panel.toggle_events();
                    };

                    ui.separator();
                })
            });
    }
}
