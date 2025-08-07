use std::sync::Arc;

use egui::{
    text::LayoutJob, vec2, Align, Button, CentralPanel as EguiCentralPanel, Color32, Context,
    Direction, Frame, Label, Layout, Margin, RichText, Sense, TextEdit, TextFormat, Ui, WidgetText,
};

use crate::{
    states::States,
    ui::main_page::central_panel::{
        request_details::RequestDetailsPanel, responses::ResponsesListPanel,
    },
};
mod request_details;
mod responses;

pub struct CentralPanel {
    request_details: RequestDetailsPanel,
    responses_list: ResponsesListPanel,
}

impl CentralPanel {
    pub fn new() -> Self {
        Self {
            request_details: RequestDetailsPanel::new(),
            responses_list: ResponsesListPanel::new(),
        }
    }

    pub fn update(&self, ctx: &Context, states: &mut States) {
        EguiCentralPanel::default()
            .frame(
                Frame::new()
                    .fill(states.style.color_main())
                    .inner_margin(Margin::same(10)),
            )
            .show(ctx, |ui| {
                if states.main_page.is_collection_selected() {
                    if states.main_page.is_request_selected() {
                        // Show Request Details when selected Request in Collection
                        self.update_request_details(ui, states);
                        return;
                    } else {
                        // Show Collection Details when selected only Collection
                        self.update_collection_details(ui, states);
                        return;
                    }
                }

                if states.main_page.is_request_selected() {
                    // Show Request Details when selected Request in Root
                    self.update_request_details(ui, states);
                    return;
                } else {
                    self.update_empty_details(ui, states)
                }
            });
    }

    fn update_collection_details(&self, ui: &mut Ui, states: &mut States) {
        ui.with_layout(Layout::top_down_justified(Align::LEFT), |ui| {
            Frame::new()
                .fill(states.style.color_main())
                .inner_margin(Margin::same(10))
                .show(ui, |ui| {
                    EntityDetailsHeaderButtons::update(ui, "Collection".into(), states);

                    ui.add_space(20.);

                    let collection_option = states.main_page.selected_collection_mut();
                    if let Some(collection) = collection_option {
                        ui.horizontal(|ui| {
                            ui.add_space(10.);
                            ui.add(Label::new("Name:").selectable(false));
                            ui.add_space(30.);
                            ui.style_mut().visuals.extreme_bg_color =
                                states.style.color_secondary();
                            let resp_collection_name = ui.add(
                                TextEdit::singleline(&mut collection.draft.name)
                                    .desired_width(ui.available_width() - 20.)
                                    .font(states.style.fonts.textedit_big())
                                    .code_editor(),
                            );

                            if resp_collection_name.changed() {
                                collection.is_changed = true;
                            };
                        });
                        ui.horizontal(|ui| {
                            ui.add_space(10.);
                            ui.add(Label::new("Description:").selectable(false));
                            ui.style_mut().visuals.extreme_bg_color =
                                states.style.color_secondary();
                            let resp_collection_desc = ui.add(
                                TextEdit::multiline(&mut collection.draft.description)
                                    .min_size(vec2(ui.available_width() - 12., 200.))
                                    .font(states.style.fonts.textedit_medium())
                                    .code_editor(),
                            );
                            if resp_collection_desc.changed() {
                                collection.is_changed = true;
                            };
                        });
                    };
                });
        });
    }
    fn update_request_details(&self, ui: &mut Ui, states: &mut States) {
        self.request_details.update(ui, states);
        self.responses_list.update(ui, states);
    }

    fn update_empty_details(&self, ui: &mut Ui, states: &mut States) {
        let mut job = LayoutJob::default();

        job.append(
            "Enjoy Frapi :)",
            0.,
            TextFormat {
                color: states.style.color_light(),
                font_id: states.style.fonts.header1(),

                ..Default::default()
            },
        );
        ui.with_layout(Layout::centered_and_justified(Direction::TopDown), |ui| {
            ui.add(Label::new(job).selectable(false))
        });
    }
}

/// Buttons block for entities - Save, Delete
pub struct EntityDetailsHeaderButtons {}

impl EntityDetailsHeaderButtons {
    fn update(ui: &mut Ui, entity: String, states: &mut States) {
        let mut header_text = LayoutJob::default();
        header_text.append(
            &format!("{entity} Details"),
            0.,
            TextFormat {
                color: Color32::DARK_GRAY,
                font_id: states.style.fonts.header2(),

                ..Default::default()
            },
        );
        ui.horizontal(|ui| {
            ui.add(Label::new(header_text).selectable(false));

            ui.add_space(ui.available_width() - 135.);

            ui.group(|ui| {
                ui.style_mut().spacing.button_padding = vec2(10., 10.);
                let entity_is_changed = states.main_page.entity_is_changed();
                if ui
                    .add(
                        Button::new(WidgetText::RichText(Arc::new(
                            RichText::new("cancel")
                                .color(if entity_is_changed {
                                    Color32::WHITE
                                } else {
                                    states.style.color_main()
                                })
                                .size(13.)
                                .monospace(),
                        )))
                        .sense(if entity_is_changed {
                            Sense::click()
                        } else {
                            Sense::empty()
                        })
                        .fill(if entity_is_changed {
                            states.style.color_danger()
                        } else {
                            states.style.color_secondary()
                        }),
                    )
                    .clicked()
                {
                    states.main_page.cancel_changes_of_selected_entity();
                };
                if ui
                    .add(
                        Button::new(WidgetText::RichText(Arc::new(
                            RichText::new("save")
                                .color(if entity_is_changed {
                                    Color32::WHITE
                                } else {
                                    states.style.color_main()
                                })
                                .monospace()
                                .size(13.),
                        )))
                        .sense(if entity_is_changed {
                            Sense::click()
                        } else {
                            Sense::empty()
                        })
                        .fill(if entity_is_changed {
                            states.style.color_success()
                        } else {
                            states.style.color_secondary()
                        }),
                    )
                    .clicked()
                {
                    states.save_selected(None);
                };
            });
        });
    }
}
