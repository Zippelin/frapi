use std::sync::Arc;

use egui::{
    text::LayoutJob, vec2, Align, Button, Color32, ComboBox, CornerRadius, FontFamily, FontId,
    FontSelection, Frame, Label, Layout, Margin, RichText, ScrollArea, Sense, Separator, TextEdit,
    TextFormat, TopBottomPanel, Ui, WidgetText,
};

use crate::{
    settings::{Method, Protocol},
    states::{
        main_page::{Entity, Header},
        States,
    },
    ui::icons::Icon,
};

pub struct RequestDetailsPanel {}

impl RequestDetailsPanel {
    pub fn new() -> Self {
        Self {}
    }

    pub fn update(&self, ui: &mut Ui, states: &mut States) {
        TopBottomPanel::top(format!(
            "request-details-{}",
            states.main_page.selected_request_salt()
        ))
        .resizable(true)
        .height_range(250.0..=400.)
        .frame(
            Frame::new()
                .fill(states.style.color_main())
                .inner_margin(Margin::same(10)),
        )
        .show_inside(ui, |ui| {
            ui.with_layout(Layout::top_down_justified(Align::LEFT), |ui| {
                self.update_header(ui, states);
                self.update_path(ui, states);
                self.update_url(ui, states);
                self.update_headers(ui, states);
            })
        });
    }

    /// Draw path line
    fn update_path(&self, ui: &mut Ui, states: &mut States) {
        ui.with_layout(Layout::left_to_right(Align::LEFT), |ui| {
            let selected_collection_idx = states.main_page.selected_entity.collection_idx;

            ui.style_mut().spacing.item_spacing = vec2(3., 1.);
            // match selected_collection {
            //     Some(val) => {
            //         if let Entity::COLLECTION(selected_collection) = &states.main_page.entities[val]
            //         {
            //             ui.label(&selected_collection.draft.name);
            //         }
            //         return;
            //     }
            //     None => {}
            // }

            let current_collection_name = match states.main_page.selected_collection() {
                Some(c) => c.draft.name.clone(),
                None => "[root] /".into(),
            };

            ui.menu_button(
                RichText::new(current_collection_name)
                    .size(15.)
                    .color(states.style.color_secondary())
                    .monospace(),
                |ui| {
                    ui.style_mut().spacing.button_padding = vec2(10., 5.);
                    if selected_collection_idx.is_some() {
                        if ui.button("[root] /").clicked() {
                            states.main_page.request_move_target.add(None);
                        }
                    }

                    for i in 0..states.main_page.entities.len() {
                        // skip collection currently owning this request
                        if selected_collection_idx.is_some()
                            && selected_collection_idx.unwrap() == i
                        {
                            continue;
                        };
                        if let Entity::COLLECTION(collection) = &states.main_page.entities[i] {
                            if ui.button(collection.draft.name.clone()).clicked() {
                                // Planning move on after loop. We cant mutate this vec during loop itself
                                states.main_page.request_move_target.add(Some(i));
                            }
                        };
                    }
                },
            );

            let request = states.main_page.selected_request_mut();

            if request.is_none() {
                return;
            };

            let request = request.unwrap();
            if ui
                .add(
                    TextEdit::singleline(&mut request.draft.name)
                        .desired_width(ui.available_width())
                        .font(FontSelection::FontId(FontId::new(
                            15.,
                            FontFamily::Monospace,
                        )))
                        .text_color(states.style.color_lighter())
                        .background_color(states.style.color_main())
                        .desired_width(ui.available_width()),
                )
                .changed()
            {
                request.is_changed = true;
            }
        });
    }

    /// Draw headers Table
    fn update_headers(&self, ui: &mut Ui, states: &mut States) {
        let request = states.main_page.selected_request_mut();

        if request.is_none() {
            return;
        };

        let request = request.unwrap();

        ui.horizontal(|ui| {
            ui.add(Label::new("Headers").selectable(false));
            ui.add(Separator::default().horizontal())
        });

        ScrollArea::vertical().show(ui, |ui| {
            ui.style_mut().spacing.item_spacing = vec2(2., 2.);
            ui.style_mut().visuals.widgets.active.corner_radius = CornerRadius::ZERO;
            ui.style_mut().visuals.widgets.inactive.corner_radius = CornerRadius::ZERO;
            ui.style_mut().visuals.widgets.hovered.corner_radius = CornerRadius::ZERO;

            let mut header_idx_for_remove = None;

            Frame::new().inner_margin(Margin::same(15)).show(ui, |ui| {
                for i in 0..request.draft.headers.len() {
                    ui.horizontal(|ui| {
                        let header_key_resp = ui.add(
                            TextEdit::singleline(&mut request.draft.headers[i].key)
                                .text_color(states.style.color_main()),
                        );
                        let header_value_reasp = ui.add(
                            TextEdit::singleline(&mut request.draft.headers[i].value)
                                .desired_width(ui.available_width() - 25.)
                                .text_color(states.style.color_main()),
                        );

                        if header_key_resp.changed() || header_value_reasp.changed() {
                            request.is_changed = true
                        }

                        if ui
                            .add(Button::new("x").fill(states.style.color_danger()))
                            .clicked()
                        {
                            header_idx_for_remove = Some(i);
                        }
                    });
                }

                // Deleting marked header
                if header_idx_for_remove.is_some() {
                    request
                        .draft
                        .headers
                        .swap_remove(header_idx_for_remove.take().unwrap());
                    request.is_changed = true;
                }

                ui.horizontal(|ui| {
                    let new_header_key_resp = ui.text_edit_singleline(&mut request.new_header.key);
                    let new_header_val_resp = ui.add(
                        TextEdit::singleline(&mut request.new_header.value)
                            .desired_width(ui.available_width() - 25.),
                    );
                    if new_header_key_resp.changed() || new_header_val_resp.changed() {
                        if request.new_header.key != "" || request.new_header.value != "" {
                            request.draft.headers.push(Header {
                                key: request.new_header.key.clone(),
                                value: request.new_header.value.clone(),
                            });

                            request.new_header.key = "".into();
                            request.new_header.value = "".into();
                            request.is_changed = true
                        }
                    }
                });
            });
        });
    }

    fn update_header(&self, ui: &mut Ui, states: &mut States) {
        let mut header_text = LayoutJob::default();
        header_text.append(
            "Request Details",
            0.,
            TextFormat {
                color: Color32::DARK_GRAY,
                font_id: FontId::new(15.0, FontFamily::Monospace),

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

    /// Draw URL group
    fn update_url(&self, ui: &mut Ui, states: &mut States) {
        ui.with_layout(Layout::top_down_justified(Align::LEFT), |ui| {
            let id_salt = states.main_page.selected_request_salt();

            let request = states.main_page.selected_request_mut();
            if request.is_none() {
                return;
            }

            let request = request.unwrap();

            ui.group(|ui| {
                ui.with_layout(Layout::left_to_right(Align::LEFT), |ui| {
                    ui.style_mut().spacing.button_padding = vec2(5., 4.);
                    ComboBox::from_id_salt(format!("selected-request-methods-{}", id_salt))
                        .selected_text(format!("{}", request.draft.method))
                        .width(70.)
                        .show_ui(ui, |ui| {
                            let method_get_resp =
                                ui.selectable_value(&mut request.draft.method, Method::GET, "GET");
                            let method_post_resp = ui.selectable_value(
                                &mut request.draft.method,
                                Method::POST,
                                "POST",
                            );
                            let method_patch_resp = ui.selectable_value(
                                &mut request.draft.method,
                                Method::PATCH,
                                "PATCH",
                            );
                            let method_put_resp =
                                ui.selectable_value(&mut request.draft.method, Method::PUT, "PUT");
                            let method_delete_resp = ui.selectable_value(
                                &mut request.draft.method,
                                Method::DELETE,
                                "DELETE",
                            );
                            if method_delete_resp.changed()
                                || method_get_resp.changed()
                                || method_patch_resp.changed()
                                || method_post_resp.changed()
                                || method_put_resp.changed()
                            {
                                request.is_changed = true;
                            }
                        });

                    ComboBox::from_id_salt(format!("selected-request-protocols-{}", id_salt))
                        .selected_text(format!("{}", request.draft.protocol))
                        .width(70.)
                        .show_ui(ui, |ui| {
                            let protocol_https_resp = ui.selectable_value(
                                &mut request.draft.protocol,
                                Protocol::HTTPS,
                                "HTTPS",
                            );
                            let protocol_http_resp = ui.selectable_value(
                                &mut request.draft.protocol,
                                Protocol::HTTP,
                                "HTTP",
                            );
                            let protocol_ws_resp = ui.selectable_value(
                                &mut request.draft.protocol,
                                Protocol::WS,
                                "WS",
                            );
                            let protocol_wss_resp = ui.selectable_value(
                                &mut request.draft.protocol,
                                Protocol::WSS,
                                "WSS",
                            );

                            if protocol_http_resp.changed()
                                || protocol_https_resp.changed()
                                || protocol_ws_resp.changed()
                                || protocol_wss_resp.changed()
                            {
                                request.is_changed = true;
                            }
                        });

                    let request_url_resp = ui.add(
                        TextEdit::singleline(&mut request.draft.uri)
                            .desired_width(ui.available_width() - 45.)
                            .text_color(states.style.color_main())
                            .background_color(states.style.color_lighter())
                            .font(FontSelection::FontId(FontId {
                                size: 16.,
                                family: FontFamily::Monospace,
                            })),
                    );

                    if request_url_resp.changed() {
                        request.is_changed = true
                    };
                    let request_executor_is_free = request.executor_is_free();

                    ui.style_mut().spacing.button_padding = vec2(10., 4.);
                    if ui
                        .add(Button::new(Icon::go()).fill(if request_executor_is_free {
                            states.style.color_success()
                        } else {
                            states.style.color_danger()
                        }))
                        .clicked()
                        && request_executor_is_free
                    {
                        request.go(Arc::clone(&states.events));
                    };
                });
            });
        });
    }
}
