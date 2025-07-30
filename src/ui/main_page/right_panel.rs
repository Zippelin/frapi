use std::sync::Arc;

use egui::{
    Align, Color32, Context, Frame, Label, Layout, Margin, RichText, ScrollArea, SidePanel,
    TextWrapMode, Ui, WidgetText,
};

use crate::states::{Event, EventData, States};

pub struct RightPanel {}

impl RightPanel {
    pub fn new() -> Self {
        Self {}
    }
    pub fn update(&mut self, ctx: &Context, states: &mut States) {
        SidePanel::right("right")
            .default_width(300.)
            .width_range(300.0..=500.)
            .frame(
                Frame::new()
                    .fill(states.style.color_secondary())
                    .inner_margin(Margin::same(5)),
            )
            .show(ctx, |ui| {
                ui.add(
                    Label::new(WidgetText::RichText(Arc::new(
                        RichText::new(format!("Events: [{}]", states.events_count()))
                            .color(states.style.color_main()),
                    )))
                    .selectable(false),
                );
                ScrollArea::vertical().show(ui, |ui| {
                    let event_guard = states.events.lock().unwrap();
                    let events = &*event_guard;
                    for i in (0..events.len()).rev() {
                        Frame::new()
                            .inner_margin(Margin::same(5))
                            .fill(states.style.color_main())
                            .show(ui, |ui| match events.get(i) {
                                Event::Info(event_data) => self.update_info_event(
                                    ui,
                                    event_data,
                                    states.style.color_secondary(),
                                ),
                                Event::Warning(event_data) => self.update_warn_event(
                                    ui,
                                    event_data,
                                    states.style.color_warning(),
                                ),
                                Event::Error(event_data) => self.update_error_event(
                                    ui,
                                    event_data,
                                    states.style.color_danger(),
                                ),
                            });
                    }
                })
            });
    }

    fn update_info_event(&self, ui: &mut Ui, event: &EventData, color: Color32) {
        ui.with_layout(Layout::top_down_justified(Align::LEFT), |ui| {
            ui.add(Label::new(WidgetText::RichText(Arc::new(
                RichText::new("event: INFO")
                    .monospace()
                    .color(color)
                    .size(14.),
            ))));
            self.update_event(ui, event, color);
        });
    }
    fn update_warn_event(&self, ui: &mut Ui, event: &EventData, color: Color32) {
        ui.with_layout(Layout::top_down_justified(Align::LEFT), |ui| {
            ui.add(Label::new(WidgetText::RichText(Arc::new(
                RichText::new("event: WARNING")
                    .monospace()
                    .color(color)
                    .size(14.),
            ))));
            self.update_event(ui, event, color);
        });
    }
    fn update_error_event(&self, ui: &mut Ui, event: &EventData, color: Color32) {
        ui.with_layout(Layout::top_down_justified(Align::LEFT), |ui| {
            ui.add(Label::new(WidgetText::RichText(Arc::new(
                RichText::new("event: ERROR")
                    .monospace()
                    .color(color)
                    .size(14.),
            ))));
            self.update_event(ui, event, color);
        });
    }

    fn update_event(&self, ui: &mut Ui, event: &EventData, color: Color32) {
        ui.horizontal(|ui| {
            ui.add(
                Label::new(WidgetText::RichText(Arc::new(
                    RichText::new("time: ").monospace().color(color).size(12.),
                )))
                .selectable(false),
            );
            ui.add(
                Label::new(WidgetText::RichText(Arc::new(
                    RichText::new(event.timestamp.format("%Y-%m-%d %H:%M:%S.%f").to_string())
                        .monospace()
                        .color(color)
                        .size(12.),
                )))
                .selectable(false)
                .wrap_mode(TextWrapMode::Wrap),
            );
        });

        ui.horizontal(|ui| {
            ui.add(
                Label::new(WidgetText::RichText(Arc::new(
                    RichText::new("message: ")
                        .monospace()
                        .color(color)
                        .size(12.),
                )))
                .selectable(false),
            );
            ui.add(
                Label::new(WidgetText::RichText(Arc::new(
                    RichText::new(event.message.clone())
                        .monospace()
                        .color(color)
                        .size(12.),
                )))
                .selectable(false)
                .wrap_mode(TextWrapMode::Wrap),
            );
        });
    }
}
