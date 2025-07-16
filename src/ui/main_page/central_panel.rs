use egui::{
    text::LayoutJob, vec2, Align, Button, CentralPanel as EguiCentralPanel, Color32, Context,
    Direction, FontFamily, FontId, Frame, Label, Layout, Margin, RichText, TextEdit, TextFormat,
    Ui,
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
            ui.add_space(10.);
            ui.horizontal(|ui| {
                let collection_option = states.main_page.selected_collection_mut();

                let mut header_text = LayoutJob::default();

                header_text.append(
                    "Collection Details",
                    0.,
                    TextFormat {
                        color: Color32::DARK_GRAY,
                        font_id: FontId::new(15.0, FontFamily::Monospace),

                        ..Default::default()
                    },
                );

                ui.horizontal(|ui| {
                    ui.add(Label::new(header_text).selectable(false));
                    if let Some(collection) = collection_option {
                        ui.style_mut().spacing.button_padding = vec2(15., 0.);
                        if collection.is_changed {
                            ui.add_space(ui.available_width() - 65.);

                            ui.add(
                                Button::new(
                                    RichText::new("Save")
                                        .heading()
                                        .monospace()
                                        .color(states.style.color_main()),
                                )
                                .fill(states.style.color_success()),
                            );
                        }
                    }
                });
            });
            ui.add_space(20.);

            let collection_option = states.main_page.selected_collection_mut();
            if let Some(collection) = collection_option {
                ui.horizontal(|ui| {
                    ui.add_space(10.);
                    ui.add(Label::new("Name:").selectable(false));
                    ui.add_space(30.);
                    let resp_collection_name = ui.add(
                        TextEdit::singleline(&mut collection.draft.name)
                            .desired_width(ui.available_width() - 20.),
                    );

                    if resp_collection_name.changed() {
                        collection.is_changed = true;
                    };
                });
                ui.horizontal(|ui| {
                    ui.add_space(10.);
                    ui.add(Label::new("Description:").selectable(false));

                    let resp_collection_desc = ui.add(
                        TextEdit::multiline(&mut collection.draft.description)
                            .min_size(vec2(ui.available_width() - 12., 200.)),
                    );
                    if resp_collection_desc.changed() {
                        collection.is_changed = true;
                    };
                });
            };
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
                font_id: FontId::new(30.0, FontFamily::Monospace),

                ..Default::default()
            },
        );
        ui.with_layout(Layout::centered_and_justified(Direction::TopDown), |ui| {
            ui.add(Label::new(job).selectable(false))
        });
    }
}
