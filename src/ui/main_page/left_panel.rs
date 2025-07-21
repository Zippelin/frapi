use std::sync::Arc;

use egui::{
    text::LayoutJob, vec2, Align, Button, Color32, Context, CornerRadius, FontFamily, FontId,
    FontSelection, Frame, Key, Label, Layout, Margin, Response, RichText, ScrollArea, SidePanel,
    Stroke, TextEdit, TextFormat, TextWrapMode, Ui, WidgetText,
};

use crate::{
    settings::{Method, Protocol},
    states::{
        main_page::{Entity, SelectedEntity},
        States, Style,
    },
    ui::icons::Icon,
};

pub struct LeftPanel {}

impl LeftPanel {
    pub fn new() -> Self {
        Self {}
    }

    pub fn update(&self, ctx: &Context, states: &mut States) {
        // Заголовок с фильтром
        SidePanel::left("left")
            .default_width(200.)
            .width_range(200.0..=300.)
            .frame(Frame::new().fill(states.style.color_main()))
            .resizable(true)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // Group of Add new Entity buttons adn filter
                    ui.group(|ui| {
                        ui.style_mut().spacing.button_padding = vec2(10., 10.);
                        ui.menu_button("New", |ui| {
                            ui.style_mut().spacing.button_padding = vec2(10., 10.);

                            if ui.button("Request").clicked() {
                                let (collection_idx, request_idx) = states.main_page.new_request();
                                if let Some(c_idx) = collection_idx {
                                    states.main_page.set_collection_fold_state(c_idx, false);
                                };
                                states
                                    .main_page
                                    .selected_entity
                                    .select_request(collection_idx, request_idx);
                                states.main_page.drop_filter();
                            };

                            if ui.button("Collection").clicked() {
                                let new_collection_idx = states.main_page.new_collection();
                                states
                                    .main_page
                                    .selected_entity
                                    .select_collection(new_collection_idx);
                                states.main_page.drop_filter();
                            };
                        });

                        ui.style_mut().visuals.widgets.active.corner_radius = CornerRadius::ZERO;
                        ui.style_mut().visuals.widgets.inactive.corner_radius = CornerRadius::ZERO;
                        ui.style_mut().visuals.widgets.hovered.corner_radius = CornerRadius::ZERO;
                        ui.style_mut().visuals.widgets.noninteractive.corner_radius =
                            CornerRadius::ZERO;

                        let filter_textedit =
                            TextEdit::singleline(&mut states.main_page.filter_text)
                                .hint_text(WidgetText::RichText(Arc::new(
                                    RichText::new("search")
                                        .color(states.style.color_secondary())
                                        .monospace()
                                        .size(15.),
                                )))
                                .text_color(states.style.color_main())
                                .char_limit(50)
                                .font(FontSelection::FontId(states.style.fonts.header2()))
                                .desired_width(ui.available_width() - 29.);

                        ui.style_mut().spacing.item_spacing = vec2(1., 0.);

                        if ui.add(filter_textedit).lost_focus()
                            && ctx.input(|i| i.key_pressed(Key::Enter))
                        {
                            states.main_page.apply_filter();
                        };

                        ui.style_mut().spacing.button_padding = vec2(6., 4.);

                        if ui
                            .add(
                                Button::new("X")
                                    .corner_radius(CornerRadius::ZERO)
                                    .fill(states.style.color_light())
                                    .stroke(Stroke::default()),
                            )
                            .clicked()
                        {
                            states.main_page.drop_filter();
                        }
                    });
                });
                ui.add_space(5.);

                // Entities list
                ui.with_layout(Layout::top_down_justified(Align::LEFT), |ui| {
                    ui.style_mut().spacing.scroll.bar_outer_margin = 2.;

                    ScrollArea::vertical()
                        .max_height(ui.available_height() - 10.)
                        .show(ui, |ui| {
                            Frame::default().show(ui, |ui| {
                                ui.style_mut().spacing.button_padding = vec2(0., 5.);

                                for i in states.main_page.filtered_entities.root_entities_idxs() {
                                    VisualEntity::new().update(ui, i, states);
                                    ui.separator();
                                }
                            });
                        });
                });
            });
    }
}

pub struct VisualEntity {}

impl VisualEntity {
    pub fn new() -> Self {
        Self {}
    }

    pub fn update(&self, ui: &mut Ui, entity_idx: usize, states: &mut States) {
        // Mark for deletion after all updates done adn data freed from ownership
        // let mut entity_for_deletion = SelectedEntity::new();

        match &mut states.main_page.entities[entity_idx] {
            Entity::COLLECTION(collection) => {
                let is_selected = states
                    .main_page
                    .selected_entity
                    .collection_is_selected(entity_idx);

                // Apply visual mark on changes
                let collection_text = if collection.is_changed {
                    &format!("{} *", &collection.draft.name)
                } else {
                    &collection.draft.name
                };

                let (fold_btn_resp, folder_btn_resp, delete_btn_resp) = self
                    .update_collection_entity(
                        ui,
                        collection.is_folded,
                        is_selected,
                        collection_text,
                        states.style.clone(),
                    );

                if fold_btn_resp.clicked() {
                    collection.is_folded = !collection.is_folded;
                };
                if folder_btn_resp.clicked() {
                    states
                        .main_page
                        .selected_entity
                        .select_collection(entity_idx);
                };
                if folder_btn_resp.double_clicked() {
                    collection.is_folded = !collection.is_folded;
                };
                if delete_btn_resp.clicked() {
                    states
                        .main_page
                        .deletion_entity
                        .select_collection(entity_idx);
                };

                let requests_idxs = states
                    .main_page
                    .filtered_entities
                    .collection_requests_idxs(entity_idx);

                if requests_idxs.is_none() {
                    return;
                };

                let requests_idxs = requests_idxs.unwrap();

                if !collection.is_folded {
                    Frame::new()
                        .fill(states.style.color_secondary())
                        .show(ui, |ui| {
                            ui.indent(format!("collection-{}-reqeusts", entity_idx), |ui| {
                                for request_idx in requests_idxs {
                                    let is_selected = states
                                        .main_page
                                        .selected_entity
                                        .request_is_selected(Some(entity_idx), request_idx);

                                    let request = &collection.requests[request_idx];

                                    let (request_btn_resp, request_delete_resp) = self
                                        .update_request_entity(
                                            ui,
                                            &request.draft.name,
                                            &request.draft.method,
                                            &request.draft.protocol,
                                            request.is_changed,
                                            is_selected,
                                            states.style.clone(),
                                        );
                                    if request_btn_resp.clicked() {
                                        states
                                            .main_page
                                            .selected_entity
                                            .select_request(Some(entity_idx), request_idx);
                                    };
                                    if request_delete_resp.clicked() {
                                        states
                                            .main_page
                                            .deletion_entity
                                            .select_request(Some(entity_idx), request_idx);
                                    }
                                }
                            });
                        });
                }
            }
            Entity::REQUEST(request) => {
                let is_selected = states
                    .main_page
                    .selected_entity
                    .request_is_selected(None, entity_idx);

                // TODO: add deletion of requests
                let (request_btn_resp, request_delete_resp) = self.update_request_entity(
                    ui,
                    &request.draft.name,
                    &request.draft.method,
                    &request.draft.protocol,
                    request.is_changed,
                    is_selected,
                    states.style.clone(),
                );

                if request_btn_resp.clicked() {
                    states
                        .main_page
                        .selected_entity
                        .select_request(None, entity_idx);
                }

                if request_delete_resp.clicked() {
                    states
                        .main_page
                        .deletion_entity
                        .select_request(None, entity_idx);
                }
            }
        };

        // If marked localy for delettion - transfer to global.
        // Later on this deletion will be processes in this frame.
        // if entity_for_deletion.is_selected() {
        //     states.main_page.deletion_entity = entity_for_deletion;
        // }
    }

    /// Draw request  item
    /// Return tuplet: (folder_tbn, delete_btn) of responses
    fn update_request_entity(
        &self,
        ui: &mut Ui,
        request_text: &String,
        method: &Method,
        protocol: &Protocol,
        is_changed: bool,
        is_selected: bool,
        style: Style,
    ) -> (Response, Response) {
        ui.with_layout(Layout::top_down_justified(Align::LEFT), |ui| {
            ui.horizontal(|ui| {
                ui.style_mut().spacing.item_spacing = vec2(0., 0.);
                ui.style_mut().spacing.button_padding = vec2(5., 8.);
                let request_details_text = self.get_request_details_text(method, protocol);

                // Truncatet name so layout wont break cos of overflow
                let allowed_letters = (ui.available_size().x - 30.) / 8.5;
                let request_text = if request_text.len() as f32 >= allowed_letters {
                    &format!(
                        "{}...",
                        request_text.split_at(allowed_letters as usize - 3).0
                    )
                } else {
                    request_text
                };

                let request_text = self.get_request_text(is_changed, request_text);

                Frame::new()
                    .inner_margin(Margin::symmetric(5, 9))
                    .fill(style.color_light())
                    .show(ui, |ui| {
                        ui.add(
                            Label::new(request_details_text)
                                .selectable(false)
                                .halign(Align::RIGHT),
                        );
                    });

                // Separate layout so inner text could align left
                let request_btn_response = ui
                    .with_layout(Layout::top_down(Align::LEFT), |ui| {
                        // Button selectable of collection
                        let request_btn = Button::selectable(is_selected, request_text)
                            .min_size(vec2(ui.available_width() - 30., 15.))
                            .corner_radius(CornerRadius::ZERO);
                        let request_btn_response = ui.add(request_btn);
                        return request_btn_response;
                    })
                    .inner;

                ui.style_mut().spacing.button_padding = vec2(5., 8.);
                // Delete Button
                let delete_btn_response = ui.add(
                    Button::new(WidgetText::RichText(Arc::new(
                        RichText::new(Icon::delete()).strong().color(Color32::WHITE),
                    )))
                    .fill(style.color_danger())
                    .corner_radius(CornerRadius::ZERO),
                );

                return (request_btn_response, delete_btn_response);
            })
        })
        .inner
        .inner
    }

    /// Draw collection folder item
    /// Return tuplet: (fold_btn, folder_btn, delete_btn) of responses
    fn update_collection_entity(
        &self,
        ui: &mut Ui,
        is_folded: bool,
        is_selected: bool,
        collection_text: &String,
        style: Style,
    ) -> (Response, Response, Response) {
        ui.with_layout(Layout::top_down_justified(Align::LEFT), |ui| {
            return ui
                .horizontal(|ui| {
                    ui.style_mut().spacing.item_spacing = vec2(0., 0.);
                    ui.style_mut().spacing.button_padding = vec2(5., 8.);

                    // Folding Button
                    let fold_btn_response = ui.add(
                        Button::new(if is_folded {
                            WidgetText::RichText(Arc::new(RichText::new(Icon::triangle_right())))
                                .monospace()
                        } else {
                            WidgetText::RichText(Arc::new(RichText::new(Icon::triangle_bottom())))
                                .monospace()
                        })
                        .fill(style.color_main())
                        .corner_radius(CornerRadius::ZERO),
                    );

                    ui.style_mut().spacing.button_padding = vec2(5., 7.);

                    // Truncatet name so layout wont break cos of overflow
                    let allowed_letters = (ui.available_size().x - 30.) / 7.;
                    let collection_text = if collection_text.len() as f32 >= allowed_letters {
                        &format!(
                            "{}...",
                            collection_text.split_at(allowed_letters as usize - 3).0
                        )
                    } else {
                        collection_text
                    };

                    let collection_folder_response = ui
                        .with_layout(Layout::top_down(Align::LEFT), |ui| {
                            // Button selectable of collection
                            let collection_folder = Button::selectable(
                                is_selected,
                                self.get_collection_text(!is_folded, &collection_text),
                            )
                            .corner_radius(CornerRadius::ZERO)
                            .min_size(vec2(ui.available_size().x - 30., 15.))
                            .wrap_mode(TextWrapMode::Truncate);

                            let collection_folder_response = ui.add(collection_folder);
                            return collection_folder_response;
                        })
                        .inner;

                    // Delete Button
                    let delete_btn_response = ui.add(
                        Button::new(WidgetText::RichText(Arc::new(
                            RichText::new(Icon::delete()).strong().color(Color32::WHITE),
                        )))
                        .fill(style.color_danger())
                        .corner_radius(CornerRadius::ZERO),
                    );

                    return (
                        fold_btn_response,
                        collection_folder_response,
                        delete_btn_response,
                    );
                })
                .inner;
        })
        .inner
    }

    /// Painted request details text
    fn get_request_details_text(&self, method: &Method, protocol: &Protocol) -> LayoutJob {
        let mut job = LayoutJob::default();
        job.break_on_newline = true;

        job.append(
            &format!(
                "{}\n{}",
                protocol.to_string().to_uppercase(),
                method.to_string().to_uppercase()
            ),
            0.,
            TextFormat {
                color: Color32::LIGHT_GREEN,
                font_id: FontId::new(6.0, FontFamily::Monospace),

                ..Default::default()
            },
        );
        job
    }

    /// Painted request item text
    fn get_request_text(&self, is_changed: bool, request_text: &String) -> LayoutJob {
        let mut job = LayoutJob::default();

        let request_text = if is_changed {
            &format!("{} *", request_text)
        } else {
            request_text
        };

        job.append(
            &request_text,
            0.,
            TextFormat {
                color: Color32::LIGHT_GRAY,
                font_id: FontId::new(13.0, FontFamily::Proportional),
                ..Default::default()
            },
        );
        job
    }

    /// Painted collection item text
    fn get_collection_text(&self, is_selected: bool, text: &String) -> LayoutJob {
        let _ = is_selected;
        let mut job = LayoutJob::default();

        // let icon = if is_selected {
        //     &Icon::folder_open()
        // } else {
        //     &Icon::folder_closed()
        // };
        // job.append(
        //     icon,
        //     10.,
        //     TextFormat {
        //         color: Color32::LIGHT_GRAY,
        //         font_id: FontId::new(15.0, FontFamily::Proportional),
        //         ..Default::default()
        //     },
        // );
        job.append(
            text,
            5.,
            TextFormat {
                color: Color32::LIGHT_GRAY,
                font_id: FontId::new(13.0, FontFamily::Proportional),
                ..Default::default()
            },
        );

        job
    }
}
