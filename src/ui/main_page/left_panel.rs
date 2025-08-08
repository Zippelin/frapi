use std::sync::Arc;

use egui::{
    text::LayoutJob, vec2, Align, Button, Color32, Context, CornerRadius, FontFamily, FontId,
    FontSelection, Frame, Key, Label, Layout, Margin, Rect, Response, RichText, ScrollArea, Sense,
    SidePanel, Stroke, TextEdit, TextFormat, TextWrapMode, Ui, WidgetText,
};

use crate::{
    settings::main_settings::entity::request_settings::{
        method_settigns::Method, protocol_settings::Protocol,
    },
    states::{States, Style},
    ui::icons::Icon,
};

/// Left Panek windget
pub struct LeftPanel {
    /// Time spend while dragging on same Collection item
    /// Used to auto open Collection if drag holded under it
    time_elapsed: usize,
}

impl LeftPanel {
    pub fn new() -> Self {
        Self { time_elapsed: 0 }
    }

    pub fn update(&mut self, ctx: &Context, states: &mut States) {
        // Заголовок с фильтром
        SidePanel::left("left")
            .default_width(200.)
            .width_range(200.0..=300.)
            .frame(Frame::new().fill(states.style.color_main()))
            .resizable(true)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // Group of Add new Entity buttons adn filter
                    Frame::new()
                        // .fill(states.style.color_secondary())
                        .inner_margin(Margin::same(5))
                        .show(ui, |ui| {
                            ui.style_mut().spacing.button_padding = vec2(10., 10.);
                            ui.menu_button(
                                states
                                    .style
                                    .fonts
                                    .menu_text("New")
                                    .color(states.style.color_secondary()),
                                |ui| {
                                    ui.style_mut().spacing.button_padding = vec2(10., 10.);

                                    if ui
                                        .button(
                                            states
                                                .style
                                                .fonts
                                                .menu_text("Request")
                                                .color(states.style.color_main()),
                                        )
                                        .clicked()
                                    {
                                        let (collection_idx, request_idx) =
                                            states.main_page.new_request();
                                        if let Some(c_idx) = collection_idx {
                                            states
                                                .main_page
                                                .set_collection_fold_state(c_idx, false);
                                        };
                                        states
                                            .main_page
                                            .selected_entity
                                            .select_request(collection_idx, request_idx);
                                        states.main_page.drop_filter();
                                    };

                                    if ui
                                        .button(
                                            states
                                                .style
                                                .fonts
                                                .menu_text("Collection")
                                                .color(states.style.color_main()),
                                        )
                                        .clicked()
                                    {
                                        let new_collection_idx = states.main_page.new_collection();
                                        states
                                            .main_page
                                            .selected_entity
                                            .select_collection(new_collection_idx);
                                        states.main_page.drop_filter();
                                    };
                                },
                            );

                            ui.style_mut().visuals.widgets.active.corner_radius =
                                CornerRadius::ZERO;
                            ui.style_mut().visuals.widgets.inactive.corner_radius =
                                CornerRadius::ZERO;
                            ui.style_mut().visuals.widgets.hovered.corner_radius =
                                CornerRadius::ZERO;
                            ui.style_mut().visuals.widgets.noninteractive.corner_radius =
                                CornerRadius::ZERO;
                            ui.style_mut().visuals.extreme_bg_color =
                                states.style.color_secondary();

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
                                    .desired_width(ui.available_width() - 30.);

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

                ui.separator();

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
                                    VisualEntity::new().update(
                                        ui,
                                        i,
                                        states,
                                        &mut self.time_elapsed,
                                    );
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

    /// Draw collection item
    fn update_collection(
        &self,
        ui: &mut Ui,
        collection_idx: usize,
        states: &mut States,
        time_elapsed: &mut usize,
    ) {
        let collection = match states.main_page.entities[collection_idx].as_collection_mut() {
            Some(val) => val,
            None => return,
        };

        let is_selected = states
            .main_page
            .selected_entity
            .collection_is_selected(collection_idx);

        self.update_drop_target(
            ui,
            states
                .main_page
                .dnd_data
                .is_drop_entity(Some(collection_idx), None),
            states.main_page.dnd_data.is_dragged(),
            &states.style,
        );

        let (fold_btn_resp, folder_btn_resp, delete_btn_resp) = self.update_collection_item(
            ui,
            collection.is_folded,
            is_selected,
            collection.is_changed,
            &collection.draft.name,
            states.style.clone(),
        );

        if fold_btn_resp.clicked() {
            collection.is_folded = !collection.is_folded;
        };

        if folder_btn_resp.drag_started() {
            states
                .main_page
                .dnd_data
                .set_dragged(Some(collection_idx), None);
        }

        if folder_btn_resp.contains_pointer() && states.main_page.dnd_data.is_dragged() {
            let drop_is_changed = states
                .main_page
                .dnd_data
                .set_dropped(Some(collection_idx), None);

            // Unfolding Collection folder if Drag over it for some time
            if drop_is_changed {
                *time_elapsed = 0;
            } else {
                *time_elapsed += 1;
            }

            if collection.is_folded && *time_elapsed == 60 {
                collection.is_folded = false;
                *time_elapsed = 0;
            }
        }

        if folder_btn_resp.clicked() {
            states
                .main_page
                .selected_entity
                .select_collection(collection_idx);
        };
        if folder_btn_resp.double_clicked() {
            collection.is_folded = !collection.is_folded;
        };
        if delete_btn_resp.clicked() {
            states
                .main_page
                .deletion_entity
                .select_collection(collection_idx);
        };

        self.update_collection_content(ui, collection_idx, states);
    }

    /// Draw collection content when unfolded
    fn update_collection_content(&self, ui: &mut Ui, collection_idx: usize, states: &mut States) {
        let collection = match states.main_page.entities[collection_idx].as_collection_mut() {
            Some(val) => val,
            None => return,
        };

        if collection.is_folded {
            return;
        }

        let requests_idxs = states
            .main_page
            .filtered_entities
            .collection_requests_idxs(collection_idx);

        if requests_idxs.is_none() {
            return;
        };

        let requests_idxs = requests_idxs.unwrap();

        if requests_idxs.len() == 0 {
            let empty_space_button = Button::selectable(
                false,
                RichText::new("no items").color(states.style.color_light()),
            )
            .min_size(vec2(ui.available_width() - 30., 15.))
            .corner_radius(CornerRadius::ZERO)
            .sense(Sense::click_and_drag())
            .image_tint_follows_text_color(true)
            .sense(Sense::DRAG);

            self.update_drop_target(
                ui,
                states
                    .main_page
                    .dnd_data
                    .is_drop_entity(Some(collection_idx), Some(0)),
                states.main_page.dnd_data.is_dragged(),
                &states.style,
            );

            let empty_space_button = Frame::new()
                .fill(states.style.color_secondary())
                .show(ui, |ui| {
                    ui.with_layout(Layout::top_down_justified(Align::Center), |ui| {
                        ui.add(empty_space_button)
                    })
                })
                .inner
                .inner;

            if empty_space_button.contains_pointer() && states.main_page.dnd_data.is_dragged() {
                states
                    .main_page
                    .dnd_data
                    .set_dropped(Some(collection_idx), Some(0));
            }
        }

        Frame::new()
            .fill(states.style.color_secondary())
            .show(ui, |ui| {
                ui.indent(format!("collection-{}-reqeusts", collection_idx), |ui| {
                    for request_idx in requests_idxs {
                        let is_selected = states
                            .main_page
                            .selected_entity
                            .request_is_selected(Some(collection_idx), request_idx);

                        let request = &collection.requests[request_idx];

                        self.update_drop_target(
                            ui,
                            states
                                .main_page
                                .dnd_data
                                .is_drop_entity(Some(collection_idx), Some(request_idx)),
                            states.main_page.dnd_data.is_dragged(),
                            &states.style,
                        );

                        let (request_btn_resp, request_delete_resp) = self.update_request_item(
                            ui,
                            &request.draft.name,
                            &request.draft.method,
                            &request.draft.protocol,
                            request.is_changed,
                            is_selected,
                            true,
                            states.style.clone(),
                        );

                        if request_btn_resp.drag_started() {
                            states
                                .main_page
                                .dnd_data
                                .set_dragged(Some(collection_idx), Some(request_idx));
                        };

                        if request_btn_resp.contains_pointer()
                            && states.main_page.dnd_data.is_dragged()
                        {
                            states
                                .main_page
                                .dnd_data
                                .set_dropped(Some(collection_idx), Some(request_idx));
                        }

                        if request_btn_resp.clicked() {
                            states
                                .main_page
                                .selected_entity
                                .select_request(Some(collection_idx), request_idx);
                        };
                        if request_delete_resp.clicked() {
                            states
                                .main_page
                                .deletion_entity
                                .select_request(Some(collection_idx), request_idx);
                        }
                    }
                });
            });
    }

    /// Draw requeus item
    fn update_request(&self, ui: &mut Ui, request_idx: usize, states: &mut States) {
        let request = match states.main_page.entities[request_idx].as_request_mut() {
            Some(val) => val,
            None => return,
        };
        let is_selected = states
            .main_page
            .selected_entity
            .request_is_selected(None, request_idx);

        // Draw Separator for drop place indication
        self.update_drop_target(
            ui,
            states
                .main_page
                .dnd_data
                .is_drop_entity(None, Some(request_idx)),
            states.main_page.dnd_data.is_dragged(),
            &states.style,
        );

        let (request_btn_resp, request_delete_resp) = self.update_request_item(
            ui,
            &request.draft.name,
            &request.draft.method,
            &request.draft.protocol,
            request.is_changed,
            is_selected,
            false,
            states.style.clone(),
        );

        // Settings dragged item
        if request_btn_resp.drag_started() {
            states
                .main_page
                .dnd_data
                .set_dragged(None, Some(request_idx));
        }

        // If DDragged setting this entity as drop target
        if request_btn_resp.contains_pointer() && states.main_page.dnd_data.is_dragged() {
            states
                .main_page
                .dnd_data
                .set_dropped(None, Some(request_idx));
        }

        if request_btn_resp.clicked() {
            states
                .main_page
                .selected_entity
                .select_request(None, request_idx);
        }

        if request_delete_resp.clicked() {
            states
                .main_page
                .deletion_entity
                .select_request(None, request_idx);
        }
    }

    pub fn update(
        &self,
        ui: &mut Ui,
        entity_idx: usize,
        states: &mut States,
        time_elapsed: &mut usize,
    ) {
        ui.style_mut().spacing.item_spacing = vec2(0., 0.);
        if states.main_page.entities[entity_idx].is_collection() {
            self.update_collection(ui, entity_idx, states, time_elapsed);
        };
        if states.main_page.entities[entity_idx].is_request() {
            self.update_request(ui, entity_idx, states);
        };

        self.update_drop(ui, states);
        self.update_dragged(ui, states);
    }

    /// Draw request  item
    /// Return tuplet: (folder_btn, delete_btn) of responses
    fn update_request_item(
        &self,
        ui: &mut Ui,
        request_text: &String,
        method: &Method,
        protocol: &Protocol,
        is_changed: bool,
        is_selected: bool,
        is_nested: bool,
        style: Style,
    ) -> (Response, Response) {
        ui.with_layout(Layout::top_down_justified(Align::LEFT), |ui| {
            ui.horizontal(|ui| {
                ui.style_mut().spacing.item_spacing = vec2(0., 0.);
                ui.style_mut().spacing.button_padding = vec2(5., 8.);
                let request_details_text = self.get_request_details_text(method, protocol, &style);

                // Truncatet name so layout wont break cos of overflow
                let allowed_letters = (ui.available_size().x - 60.) / 8.5;
                let request_text = if request_text.len() as f32 >= allowed_letters {
                    &format!(
                        "{}...",
                        request_text.split_at(allowed_letters as usize - 5).0
                    )
                } else {
                    request_text
                };

                let request_text =
                    self.get_request_text(is_changed, request_text, is_nested, &style);

                Frame::new()
                    .inner_margin(Margin::symmetric(5, 9))
                    .fill(style.color_light())
                    .show(ui, |ui| {
                        ui.add_sized(
                            vec2(20., ui.available_height()),
                            Label::new(request_details_text)
                                .selectable(false)
                                .halign(Align::RIGHT),
                        )
                    });
                // Separate layout so inner text could align left
                let request_btn_response = ui
                    .with_layout(Layout::top_down(Align::LEFT), |ui| {
                        // Button selectable of collection
                        let request_btn = Button::selectable(is_selected, request_text)
                            .min_size(vec2(ui.available_width() - 30., 15.))
                            .corner_radius(CornerRadius::ZERO)
                            .sense(Sense::click_and_drag())
                            .image_tint_follows_text_color(true);

                        ui.add(request_btn)
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
    fn update_collection_item(
        &self,
        ui: &mut Ui,
        is_folded: bool,
        is_selected: bool,
        is_changed: bool,
        collection_text: &String,
        style: Style,
    ) -> (Response, Response, Response) {
        ui.with_layout(Layout::top_down_justified(Align::LEFT), |ui| {
            let result = ui
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
                    let allowed_letters = (ui.available_size().x - 60.) / 8.;
                    let collection_text = if collection_text.len() as f32 >= allowed_letters {
                        &format!(
                            "{}...",
                            collection_text.split_at(allowed_letters as usize - 5).0
                        )
                    } else {
                        collection_text
                    };

                    let collection_folder_response = ui
                        .with_layout(Layout::top_down(Align::LEFT), |ui| {
                            // Button selectable of collection
                            let collection_folder = Button::selectable(
                                is_selected,
                                self.get_collection_text(is_changed, &collection_text, &style),
                            )
                            .corner_radius(CornerRadius::ZERO)
                            .min_size(vec2(ui.available_size().x - 30., 15.))
                            .wrap_mode(TextWrapMode::Truncate)
                            .sense(Sense::click_and_drag());

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

            // If Collection unfolded draw shadow-like separator
            if !is_folded {
                ui.style_mut().spacing.item_spacing = vec2(0., 0.);

                let color = Color32::BLACK.linear_multiply(0.2);

                Frame::new()
                    .outer_margin(Margin::same(1))
                    .fill(color)
                    .show(ui, |ui| {
                        ui.style_mut().visuals.widgets.noninteractive.bg_stroke =
                            Stroke::new(0., color);
                        ui.separator();
                    });
            }

            result
        })
        .inner
    }

    /// Painted request details text
    fn get_request_details_text(
        &self,
        method: &Method,
        protocol: &Protocol,
        style: &Style,
    ) -> LayoutJob {
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
                color: style.color_success(),
                font_id: FontId::new(6.0, FontFamily::Monospace),

                ..Default::default()
            },
        );
        job
    }

    /// Painted request item text
    fn get_request_text(
        &self,
        is_changed: bool,
        request_text: &String,
        is_nested: bool,
        style: &Style,
    ) -> LayoutJob {
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
                color: if is_nested {
                    style.color_main()
                } else {
                    style.color_secondary()
                },
                font_id: FontId::new(13.0, FontFamily::Monospace),
                ..Default::default()
            },
        );
        job
    }

    /// Painted collection item text
    fn get_collection_text(
        &self,
        is_changed: bool,
        collection_text: &String,
        style: &Style,
    ) -> LayoutJob {
        let mut job = LayoutJob::default();

        let collection_text = if is_changed {
            &format!("{} *", collection_text)
        } else {
            collection_text
        };

        job.append(
            collection_text,
            5.,
            TextFormat {
                color: style.color_secondary(),
                font_id: FontId::new(13.0, FontFamily::Monospace),
                ..Default::default()
            },
        );

        job
    }

    // TODO: complete drop logic
    fn update_drop(&self, ui: &mut Ui, states: &mut States) {
        if !states.main_page.dnd_data.is_dragged() {
            return;
        }

        ui.ctx().input(|r| {
            if r.pointer.button_released(egui::PointerButton::Primary) {
                states.main_page.dnd_data.finalize();
            }
        });
    }

    /// Draw dragget item
    fn update_dragged(&self, ui: &mut Ui, states: &mut States) {
        if states.main_page.dnd_data.is_dragged() {
            let pointer_pos = match ui.ctx().pointer_hover_pos() {
                Some(val) => val,
                None => {
                    states.main_page.dnd_data.clear();
                    return;
                }
            };

            let size = vec2(ui.available_width(), 40.);
            let topmost_layer_on_pos = match ui.ctx().layer_id_at(pointer_pos) {
                Some(val) => val,
                None => {
                    states.main_page.dnd_data.clear();
                    return;
                }
            };
            let mut painter = ui.ctx().layer_painter(topmost_layer_on_pos);

            painter.set_opacity(0.5);
            painter.rect_filled(
                Rect::from_center_size(pointer_pos, size),
                2.0,
                states.style.color_secondary(),
            );

            if let Some(selected_text) = states.main_page.get_dragged_entity_text() {
                painter.text(
                    pointer_pos,
                    egui::Align2::CENTER_CENTER,
                    &selected_text,
                    egui::FontId::default(),
                    states.style.color_main(),
                );
            }
        }
    }

    /// Draw Separator for drop place indication
    fn update_drop_target(
        &self,
        ui: &mut Ui,
        is_drop_target: bool,
        is_dragged: bool,
        style: &Style,
    ) {
        if !is_drop_target || !is_dragged {
            return;
        }

        Frame::new()
            .fill(style.color_lighter())
            .inner_margin(Margin::same(1))
            .show(ui, |ui| {
                ui.style_mut().visuals.widgets.noninteractive.bg_stroke =
                    (1.0, style.color_lighter()).into();
                ui.separator();
            });
        // };
    }
}
