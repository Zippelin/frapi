use std::sync::Arc;

use egui::{
    vec2, Align, Button, ComboBox, CornerRadius, FontFamily, FontId, FontSelection, Frame, Label,
    Layout, Margin, RichText, ScrollArea, Separator, TextEdit, TopBottomPanel, Ui, WidgetText,
};
use rfd::FileDialog;

use crate::{
    settings::main_settings::entity::request_settings::{
        method_settigns::Method, protocol_settings::Protocol,
    },
    states::{
        main_page::{
            entity::Entity,
            generics::{CountedText, Header},
            request::{
                default_ws_headers,
                request_data::{BodyFromData, FormFieldType},
                HttpVersion, RequestBodyDetails, RequestDetails, RequestHeaders, RequestSetup,
            },
        },
        States, Style,
    },
    ui::{icons::Icon, main_page::central_panel::EntityDetailsHeaderButtons},
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
        .min_height(250.)
        .frame(
            Frame::new()
                .fill(states.style.color_main())
                .inner_margin(Margin::same(10)),
        )
        .show_inside(ui, |ui| {
            ui.with_layout(Layout::top_down_justified(Align::LEFT), |ui| {
                EntityDetailsHeaderButtons::update(ui, "Request".into(), states);
                self.update_path(ui, states);
                self.update_url(ui, states);

                // Button to select request part to change
                ui.horizontal(|ui| {
                    let request = states.main_page.selected_request_mut();
                    if request.is_none() {
                        return;
                    };

                    let request = request.unwrap();

                    if request.draft.protocot_is_ws() {
                        ui.radio_value(
                            &mut request.visible_details,
                            RequestDetails::Message,
                            "Message",
                        );
                    }

                    if request.draft.protocot_is_http() {
                        ui.radio_value(
                            &mut request.visible_details,
                            RequestDetails::QueryParams,
                            "Query Params",
                        );
                    }

                    ui.radio_value(
                        &mut request.visible_details,
                        RequestDetails::Header,
                        "Headers",
                    );

                    if request.draft.protocot_is_http() {
                        ui.radio_value(&mut request.visible_details, RequestDetails::Body, "Body");
                    }

                    ui.radio_value(&mut request.visible_details, RequestDetails::Setup, "Setup");

                    ui.add(Separator::default().horizontal());
                });

                ui.add_space(10.);

                // Select view of request part
                if let Some(request) = states.main_page.selected_request() {
                    match request.visible_details {
                        RequestDetails::Header => self.update_headers(ui, states),
                        RequestDetails::Body => self.update_body(ui, states),
                        RequestDetails::QueryParams => self.update_query_params(ui, states),
                        RequestDetails::Message => self.update_message(ui, states),
                        RequestDetails::Setup => self.update_setup(ui, states),
                    };
                }
            })
        });
    }

    /// Draw Setup Settings
    fn update_setup(&self, ui: &mut Ui, states: &mut States) {
        ScrollArea::vertical().show(ui, |ui| {
            let request = states.main_page.selected_request_mut().unwrap();

            match request.draft.protocol {
                Protocol::HTTP | Protocol::HTTPS => self.update_setup_http(ui, states),

                Protocol::WS | Protocol::WSS => self.update_setup_ws(ui, states),
            }
        });
    }

    fn update_setup_http(&self, ui: &mut Ui, states: &mut States) {
        let request = states.main_page.selected_request_mut().unwrap();
        let setup = request.draft.setup.http_mut().unwrap();
        Frame::new().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.add(Label::new(states.style.fonts.label_text("HTTP version:")));

                ui.add_space(50.);
                ui.menu_button(setup.http_version.to_string(), |ui| {
                    ui.style_mut().spacing.button_padding = vec2(5., 5.);

                    if setup.http_version != HttpVersion::AUTO {
                        if ui.button("AUTO").clicked() {
                            setup.http_version = HttpVersion::AUTO;
                            request.is_changed = true
                        }
                    }

                    if setup.http_version != HttpVersion::HTTPv1 {
                        if ui.button("HTTPv1").clicked() {
                            setup.http_version = HttpVersion::HTTPv1;
                            request.is_changed = true
                        }
                    }

                    if setup.http_version != HttpVersion::HTTPv2 {
                        if ui.button("HTTPv2").clicked() {
                            setup.http_version = HttpVersion::HTTPv2;
                            request.is_changed = true
                        }
                    }
                });

                ui.add_space(ui.available_width());
            })
        });

        ui.add_space(10.);

        Frame::new().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.add(Label::new(states.style.fonts.label_text("Use cookies:")));

                ui.add_space(60.);
                ui.style_mut().spacing.button_padding = vec2(5., 5.);
                ui.style_mut().spacing.item_spacing = vec2(1., 10.);

                if ui
                    .add(
                        Button::new(WidgetText::RichText(Arc::new(
                            RichText::new("true").color(states.style.color_main()),
                        )))
                        .corner_radius(CornerRadius::ZERO)
                        .fill(if setup.use_cookies {
                            states.style.color_lighter()
                        } else {
                            states.style.color_light()
                        }),
                    )
                    .clicked()
                {
                    setup.use_cookies = true;
                    request.is_changed = true
                };
                if ui
                    .add(
                        Button::new(WidgetText::RichText(Arc::new(
                            RichText::new("false").color(states.style.color_main()),
                        )))
                        .corner_radius(CornerRadius::ZERO)
                        .fill(if !setup.use_cookies {
                            states.style.color_lighter()
                        } else {
                            states.style.color_light()
                        }),
                    )
                    .clicked()
                {
                    setup.use_cookies = false;
                    request.is_changed = true
                };
            });
        });

        ui.add_space(10.);

        Frame::new().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.add(
                    Label::new(states.style.fonts.label_text("Use redirects:")).selectable(false),
                );

                ui.add_space(44.);

                ui.style_mut().spacing.button_padding = vec2(5., 5.);
                ui.style_mut().spacing.item_spacing = vec2(1., 10.);

                if ui
                    .add(
                        Button::new(WidgetText::RichText(Arc::new(
                            RichText::new("true").color(states.style.color_main()),
                        )))
                        .corner_radius(CornerRadius::ZERO)
                        .fill(if setup.use_redirects {
                            states.style.color_lighter()
                        } else {
                            states.style.color_light()
                        }),
                    )
                    .clicked()
                {
                    setup.use_redirects = true;
                    request.is_changed = true
                };
                if ui
                    .add(
                        Button::new(WidgetText::RichText(Arc::new(
                            RichText::new("false").color(states.style.color_main()),
                        )))
                        .corner_radius(CornerRadius::ZERO)
                        .fill(if !setup.use_redirects {
                            states.style.color_lighter()
                        } else {
                            states.style.color_light()
                        }),
                    )
                    .clicked()
                {
                    setup.use_redirects = false;
                    request.is_changed = true
                };
            });
        });

        ui.add_space(10.);

        Frame::new().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.add(Label::new(
                    states.style.fonts.label_text("Redirects amount:"),
                ));

                ui.add_space(20.);

                let initial_value = setup.redirects_amount.clone();
                if ui
                    .add(
                        TextEdit::singleline(&mut setup.redirects_amount)
                            .char_limit(2)
                            .desired_width(20.),
                    )
                    .changed()
                {
                    match setup.redirects_amount.parse::<usize>() {
                        Ok(val) => {
                            setup.redirects_amount = val.to_string();
                            request.is_changed = true
                        }
                        Err(_) => {
                            if setup.redirects_amount.len() == 0 {
                                setup.redirects_amount = "0".into();
                                request.is_changed = true
                            } else {
                                setup.redirects_amount = initial_value;
                            }
                        }
                    }
                };
            });
        });
    }

    fn update_setup_ws(&self, ui: &mut Ui, states: &mut States) {
        let request = states.main_page.selected_request_mut().unwrap();

        let setup = request.draft.setup.ws_mut().unwrap();

        Frame::new().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.add(Label::new(
                    states.style.fonts.label_text("Reconnection timeout (ms):"),
                ));

                ui.add_space(25.);

                let initial_value = setup.reconnection_timeout.clone();
                if ui
                    .add(
                        TextEdit::singleline(&mut setup.reconnection_timeout)
                            .char_limit(10)
                            .desired_width(70.),
                    )
                    .changed()
                {
                    match setup.reconnection_timeout.parse::<usize>() {
                        Ok(val) => {
                            setup.reconnection_timeout = val.to_string();
                            request.is_changed = true
                        }
                        Err(_) => {
                            if setup.reconnection_timeout.len() == 0 {
                                setup.reconnection_timeout = "0".into();
                                request.is_changed = true
                            } else {
                                setup.reconnection_timeout = initial_value;
                            }
                        }
                    }
                };
            });
        });

        ui.add_space(10.);

        Frame::new().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.add(Label::new(
                    states.style.fonts.label_text("Reconnection attempts:"),
                ));

                ui.add_space(20.);

                let initial_value = setup.reconnection_attempts.clone();
                if ui
                    .add(
                        TextEdit::singleline(&mut setup.reconnection_attempts)
                            .char_limit(2)
                            .desired_width(20.),
                    )
                    .changed()
                {
                    match setup.reconnection_attempts.parse::<usize>() {
                        Ok(val) => {
                            setup.reconnection_attempts = val.to_string();
                            request.is_changed = true
                        }
                        Err(_) => {
                            if setup.reconnection_attempts.len() == 0 {
                                setup.reconnection_attempts = "0".into();
                                request.is_changed = true
                            } else {
                                setup.reconnection_attempts = initial_value;
                            }
                        }
                    }
                };
            });
        });
    }

    fn update_message(&self, ui: &mut Ui, states: &mut States) {
        ui.group(|ui| {
            let request = states.main_page.selected_request_mut().unwrap();
            let send_btn_response = ui.horizontal(|ui| {
                ui.style_mut().spacing.button_padding = vec2(5., 5.);
                if ui.button("Prettier").clicked() {
                    request.prettier_ws_message();
                };

                ui.add_space(ui.available_width() - 30.);

                ui.add(Button::new("Send").fill(states.style.color_success()))
                    .clicked()
            });
            if send_btn_response.inner {
                request.go(Arc::clone(&states.events), false);
            };
            if self
                .update_counted_textedit(ui, &mut request.draft.message, &states.style)
                .is_some()
            {
                request.is_changed = true;
            };
        });
    }

    /// Generic Draw of text edit with counted lines
    fn update_counted_textedit(
        &self,
        ui: &mut Ui,
        counted_text: &mut CountedText,
        style: &Style,
    ) -> Option<()> {
        let mut is_changed = None;
        ScrollArea::vertical().show(ui, |ui| {
            Frame::new().inner_margin(Margin::same(5)).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.style_mut().visuals.extreme_bg_color = style.color_secondary();
                    let textedit = TextEdit::multiline(&mut counted_text.message)
                        .min_size(vec2(ui.available_width() - 20., ui.available_height()))
                        .code_editor()
                        .text_color(style.color_main());

                    ui.style_mut().spacing.item_spacing = vec2(2., 3.);
                    ui.vertical(|ui| {
                        Frame::default()
                            .fill(style.color_lighter())
                            .inner_margin(Margin::same(4))
                            .show(ui, |ui| {
                                if counted_text.rows == 0 {
                                    let text = WidgetText::RichText(Arc::new(
                                        RichText::new("1").size(10.).color(style.color_secondary()),
                                    ));
                                    ui.add(Label::new(text).selectable(false).halign(Align::RIGHT));
                                } else {
                                    for i in 0..counted_text.rows {
                                        let text = WidgetText::RichText(Arc::new(
                                            RichText::new((i + 1).to_string())
                                                .size(10.)
                                                .color(style.color_secondary()),
                                        ));
                                        ui.add(
                                            Label::new(text).selectable(false).halign(Align::RIGHT),
                                        );
                                    }
                                };
                            })
                    });
                    if ui.add(textedit).changed() {
                        counted_text.update_rows();
                        is_changed = Some(());
                    };
                })
            });
        });
        is_changed
    }

    fn update_query_params(&self, ui: &mut Ui, states: &mut States) {
        let request = states.main_page.selected_request_mut();

        if request.is_none() {
            return;
        };

        let request = request.unwrap();

        if self
            .update_generic_headers_table(
                ui,
                &mut request.draft.query_params,
                Some(&mut request.new_header),
                &states.style,
            )
            .is_some()
        {
            request.is_changed = true;
            request.draft.contruct_url();
        };
    }

    /// Draw body part
    fn update_body(&self, ui: &mut Ui, states: &mut States) {
        let request = states.main_page.selected_request_mut().unwrap();
        ui.group(|ui| {
            ui.horizontal(|ui| {
                if ui
                    .add(
                        Button::new("Raw")
                            .selected(request.visible_body == RequestBodyDetails::Raw),
                    )
                    .clicked()
                {
                    request.visible_body = RequestBodyDetails::Raw
                }

                if ui
                    .add(
                        Button::new("Form Data")
                            .selected(request.visible_body == RequestBodyDetails::FormData),
                    )
                    .clicked()
                {
                    request.visible_body = RequestBodyDetails::FormData
                }
                if ui
                    .add(
                        Button::new("Binary")
                            .selected(request.visible_body == RequestBodyDetails::Binary),
                    )
                    .clicked()
                {
                    request.visible_body = RequestBodyDetails::Binary
                }
                ui.add_space(ui.available_width());
            });
        });

        match request.visible_body {
            RequestBodyDetails::Raw => self.update_body_raw(ui, states),
            RequestBodyDetails::FormData => self.update_body_formdata(ui, states),
            RequestBodyDetails::Binary => self.update_body_binary(ui, states),
        }
    }

    /// Draw binary file select
    fn update_body_binary(&self, ui: &mut Ui, states: &mut States) {
        ui.horizontal(|ui| {
            ui.group(|ui| {
                let request = states.main_page.selected_request_mut().unwrap();
                ui.style_mut().visuals.extreme_bg_color = states.style.color_secondary();
                ui.add(
                    TextEdit::singleline(&mut request.draft.body.binary_path)
                        .desired_width(ui.available_width() - 65.)
                        .text_color(states.style.color_main()),
                );

                if ui.add(Button::new("Browse")).clicked() {
                    let file = FileDialog::new().set_directory("./").pick_file();
                    if let Some(file_path) = file {
                        request.draft.body.binary_path = file_path.to_string_lossy().to_string();
                        request.is_changed = true;
                    }
                }
            })
        });
    }

    /// Draw form data
    fn update_body_formdata(&self, ui: &mut Ui, states: &mut States) {
        let request = states.main_page.selected_request_mut().unwrap();
        if self
            .update_form_data_table(
                ui,
                &mut request.draft.body.form_data,
                Some(&mut request.new_body_form_field),
                &states.style,
            )
            .is_some()
        {
            request.is_changed = true
        };
    }

    /// Draw body part
    fn update_body_raw(&self, ui: &mut Ui, states: &mut States) {
        let request = states.main_page.selected_request_mut().unwrap();
        if self
            .update_counted_textedit(ui, &mut request.draft.body.raw, &states.style)
            .is_some()
        {
            request.is_changed = true;
        }
    }

    /// Draw path line
    fn update_path(&self, ui: &mut Ui, states: &mut States) {
        ui.with_layout(Layout::left_to_right(Align::LEFT), |ui| {
            let selected_collection_idx = states.main_page.selected_entity.collection_idx;

            ui.style_mut().spacing.item_spacing = vec2(3., 1.);

            let current_collection_name = match states.main_page.selected_collection() {
                Some(c) => format!("{} / ", c.draft.name.clone()),
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
                            if ui
                                .button(format!("{} / ", collection.draft.name.clone()))
                                .clicked()
                            {
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

    /// Draw generic table of headers.
    /// If no new_value provided - all data is read only
    /// Return is_changed state - Some() - heave changes, None - no changes.
    fn update_form_data_table(
        &self,
        ui: &mut Ui,
        items: &mut Vec<BodyFromData>,
        new_value: Option<&mut BodyFromData>,
        style: &Style,
    ) -> Option<()> {
        let mut is_changes = None;
        ScrollArea::vertical().show(ui, |ui| {
            ui.style_mut().spacing.item_spacing = vec2(2., 2.);
            ui.style_mut().visuals.widgets.active.corner_radius = CornerRadius::ZERO;
            ui.style_mut().visuals.widgets.inactive.corner_radius = CornerRadius::ZERO;
            ui.style_mut().visuals.widgets.hovered.corner_radius = CornerRadius::ZERO;

            let mut header_idx_for_remove = None;

            Frame::new()
                .inner_margin(Margin::same(10).left_top())
                .show(ui, |ui| {
                    for i in 0..items.len() {
                        ui.horizontal(|ui| {
                            ui.style_mut().visuals.extreme_bg_color = style.color_secondary();
                            let header_key_resp = ui.add(
                                TextEdit::singleline(&mut items[i].key)
                                    .text_color(style.color_main())
                                    .font(style.fonts.textedit_small()),
                            );

                            ComboBox::from_id_salt(format!("form-data-field-type-{i}"))
                                .selected_text(items[i].field_type.to_string())
                                .width(50.)
                                .show_ui(ui, |ui| {
                                    let field_type_text = ui.selectable_value(
                                        &mut items[i].field_type,
                                        FormFieldType::Text,
                                        "Text",
                                    );
                                    let field_type_file = ui.selectable_value(
                                        &mut items[i].field_type,
                                        FormFieldType::File,
                                        "File",
                                    );

                                    if field_type_file.changed() || field_type_text.changed() {
                                        is_changes = Some(());
                                        items[i].value = "".into();
                                    }
                                });

                            let header_value_reasp = ui.add(
                                TextEdit::singleline(&mut items[i].value)
                                    .desired_width(ui.available_width() - 25.)
                                    .text_color(style.color_main())
                                    .font(style.fonts.textedit_small()),
                            );

                            if header_key_resp.changed() || header_value_reasp.changed() {
                                is_changes = Some(())
                            }

                            if new_value.is_some() {
                                if ui
                                    .add(Button::new("x").fill(style.color_danger()))
                                    .clicked()
                                {
                                    header_idx_for_remove = Some(i);
                                }
                            }
                        });
                    }

                    if new_value.is_some() {
                        // Deleting marked header
                        if header_idx_for_remove.is_some() {
                            items.swap_remove(header_idx_for_remove.take().unwrap());
                            // request.is_changed = true;
                            is_changes = Some(())
                        }
                    }

                    if let Some(new_value) = new_value {
                        ui.horizontal(|ui| {
                            ui.style_mut().visuals.extreme_bg_color = style.color_secondary();
                            let new_header_key_resp = ui.add(
                                TextEdit::singleline(&mut new_value.key).hint_text("new form key"),
                            );

                            ComboBox::from_id_salt(format!("form-data-field-type-new"))
                                .selected_text(new_value.field_type.to_string())
                                .width(50.)
                                .show_ui(ui, |ui| {
                                    let field_type_text = ui.selectable_value(
                                        &mut new_value.field_type,
                                        FormFieldType::Text,
                                        "Text",
                                    );
                                    let field_type_file = ui.selectable_value(
                                        &mut new_value.field_type,
                                        FormFieldType::File,
                                        "File",
                                    );

                                    if field_type_file.changed() || field_type_text.changed() {
                                        is_changes = Some(())
                                    }
                                });

                            let new_header_val_resp = ui.add(
                                TextEdit::singleline(&mut new_value.value)
                                    .desired_width(ui.available_width() - 25.)
                                    .hint_text("new form value"),
                            );
                            if new_header_key_resp.changed() || new_header_val_resp.changed() {
                                if new_value.key != "" || new_value.value != "" {
                                    items.push(BodyFromData {
                                        key: new_value.key.clone(),
                                        value: new_value.value.clone(),
                                        field_type: new_value.field_type.clone(),
                                    });

                                    new_value.key = "".into();
                                    new_value.value = "".into();
                                    new_value.field_type = FormFieldType::Text;
                                    is_changes = Some(())
                                }
                            }
                        });
                    }
                });
        });
        is_changes
    }

    /// Draw Form Data Table.
    /// If no new_value provided - all data is read only
    /// Return is_changed state - Some() - heave changes, None - no changes.
    fn update_generic_headers_table(
        &self,
        ui: &mut Ui,
        items: &mut Vec<Header>,
        new_value: Option<&mut Header>,
        style: &Style,
    ) -> Option<()> {
        let mut is_changes = None;
        ScrollArea::vertical().show(ui, |ui| {
            ui.style_mut().spacing.item_spacing = vec2(2., 2.);
            ui.style_mut().visuals.widgets.active.corner_radius = CornerRadius::ZERO;
            ui.style_mut().visuals.widgets.inactive.corner_radius = CornerRadius::ZERO;
            ui.style_mut().visuals.widgets.hovered.corner_radius = CornerRadius::ZERO;

            let mut header_idx_for_remove = None;

            Frame::new()
                .inner_margin(Margin::same(10).left_top())
                .show(ui, |ui| {
                    for i in 0..items.len() {
                        ui.horizontal(|ui| {
                            ui.style_mut().visuals.extreme_bg_color = style.color_secondary();
                            let header_key_resp = ui.add(
                                TextEdit::singleline(&mut items[i].key)
                                    .text_color(style.color_main())
                                    .font(style.fonts.textedit_small()),
                            );
                            let header_value_reasp = ui.add(
                                TextEdit::singleline(&mut items[i].value)
                                    .desired_width(ui.available_width() - 25.)
                                    .text_color(style.color_main())
                                    .font(style.fonts.textedit_small()),
                            );

                            if header_key_resp.changed() || header_value_reasp.changed() {
                                // request.is_changed = true
                                is_changes = Some(())
                            }

                            if new_value.is_some() {
                                if ui
                                    .add(Button::new("x").fill(style.color_danger()))
                                    .clicked()
                                {
                                    header_idx_for_remove = Some(i);
                                }
                            }
                        });
                    }

                    if new_value.is_some() {
                        // Deleting marked header
                        if header_idx_for_remove.is_some() {
                            items.swap_remove(header_idx_for_remove.take().unwrap());
                            // request.is_changed = true;
                            is_changes = Some(())
                        }
                    }

                    if let Some(new_value) = new_value {
                        ui.horizontal(|ui| {
                            ui.style_mut().visuals.extreme_bg_color = style.color_secondary();
                            let new_header_key_resp = ui.text_edit_singleline(&mut new_value.key);
                            let new_header_val_resp = ui.add(
                                TextEdit::singleline(&mut new_value.value)
                                    .desired_width(ui.available_width() - 25.),
                            );
                            if new_header_key_resp.changed() || new_header_val_resp.changed() {
                                if new_value.key != "" || new_value.value != "" {
                                    items.push(Header {
                                        key: new_value.key.clone(),
                                        value: new_value.value.clone(),
                                    });

                                    new_value.key = "".into();
                                    new_value.value = "".into();
                                    // request.is_changed = true
                                    is_changes = Some(())
                                }
                            }
                        });
                    }
                });
        });
        is_changes
    }

    /// Draw headers Table
    fn update_headers(&self, ui: &mut Ui, states: &mut States) {
        let request = states.main_page.selected_request_mut();

        if request.is_none() {
            return;
        };

        let request = request.unwrap();

        if [Protocol::WS, Protocol::WSS].contains(&request.draft.protocol) {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    if ui
                        .add(Button::new("Custom").fill(
                            if request.visible_headers == RequestHeaders::Custom {
                                states.style.color_light()
                            } else {
                                states.style.color_main()
                            },
                        ))
                        .clicked()
                    {
                        request.visible_headers = RequestHeaders::Custom
                    };

                    if ui
                        .add(Button::new("Default").fill(
                            if request.visible_headers == RequestHeaders::Default {
                                states.style.color_light()
                            } else {
                                states.style.color_main()
                            },
                        ))
                        .clicked()
                    {
                        request.visible_headers = RequestHeaders::Default
                    };
                    ui.add_space(ui.available_width());
                });
            });
        };

        if request.visible_headers == RequestHeaders::Custom {
            if self
                .update_generic_headers_table(
                    ui,
                    &mut request.draft.headers,
                    Some(&mut request.new_header),
                    &states.style,
                )
                .is_some()
            {
                request.is_changed = true
            };
        } else {
            self.update_generic_headers_table(ui, &mut default_ws_headers(), None, &states.style);
        }
    }

    /// Draw URL group
    fn update_url(&self, ui: &mut Ui, states: &mut States) {
        let id_salt = states.main_page.selected_request_salt();

        let request = states.main_page.selected_request_mut();
        if request.is_none() {
            return;
        }

        let request = request.unwrap();
        let executos_is_free = request.executor_is_free();

        ui.with_layout(Layout::top_down_justified(Align::LEFT), |ui| {
            ui.group(|ui| {
                ui.with_layout(Layout::left_to_right(Align::LEFT), |ui| {
                    ui.style_mut().spacing.button_padding = vec2(5., 4.);

                    if request.draft.protocot_is_http() {
                        if executos_is_free {
                            ComboBox::from_id_salt(format!("selected-request-methods-{}", id_salt))
                                .selected_text(format!("{}", request.draft.method))
                                .width(70.)
                                .show_ui(ui, |ui| {
                                    let method_get_resp = ui.selectable_value(
                                        &mut request.draft.method,
                                        Method::GET,
                                        "GET",
                                    );
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
                                    let method_put_resp = ui.selectable_value(
                                        &mut request.draft.method,
                                        Method::PUT,
                                        "PUT",
                                    );
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
                        } else {
                            ui.horizontal(|ui| {
                                ui.add(
                                    Button::new(WidgetText::RichText(Arc::new(
                                        RichText::new(format!(
                                            "{} {}",
                                            request.draft.method,
                                            "    "
                                                .repeat(6 - request.draft.method.to_string().len())
                                        ))
                                        // .size(13.)
                                        .strong(),
                                    )))
                                    .min_size(vec2(70., 20.)),
                                );
                            });
                        }
                    }

                    if executos_is_free {
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

                                if protocol_https_resp.changed() || protocol_http_resp.changed() {
                                    request.draft.setup = RequestSetup::default()
                                }

                                if protocol_ws_resp.changed() || protocol_wss_resp.changed() {
                                    request.draft.setup = RequestSetup::default_ws()
                                }

                                if protocol_http_resp.changed()
                                    || protocol_https_resp.changed()
                                    || protocol_ws_resp.changed()
                                    || protocol_wss_resp.changed()
                                {
                                    request.is_changed = true;
                                }
                            });
                    } else {
                        ui.horizontal(|ui| {
                            ui.style_mut().spacing.button_padding = vec2(0., 0.);
                            ui.add_sized(
                                vec2(70., 20.),
                                Button::new(WidgetText::RichText(Arc::new(
                                    RichText::new(format!(
                                        "{} {}",
                                        request.draft.protocol,
                                        "    ".repeat(5 - request.draft.method.to_string().len())
                                    ))
                                    // .size(13.)
                                    .strong(),
                                )))
                                .min_size(vec2(70., 20.)),
                            );
                        });
                    };

                    if executos_is_free {
                        let request_url_resp = ui.add(
                            TextEdit::singleline(&mut request.draft.uri)
                                .desired_width(ui.available_width() - 45.)
                                .text_color(states.style.color_lighter())
                                .background_color(states.style.color_light())
                                .font(states.style.fonts.textedit_big()),
                        );

                        if request_url_resp.changed() {
                            request.parse_url();
                            request.is_changed = true
                        };
                    } else {
                        ui.add(
                            TextEdit::singleline(&mut request.draft.uri.clone())
                                .desired_width(ui.available_width() - 45.)
                                .text_color(states.style.color_lighter())
                                .background_color(states.style.color_secondary())
                                .font(states.style.fonts.textedit_big()),
                        );
                    }

                    ui.style_mut().spacing.button_padding = vec2(10., 4.);
                    let execute_request_btn_resp =
                        ui.add(Button::new(Icon::go()).fill(if executos_is_free {
                            states.style.color_success()
                        } else {
                            states.style.color_danger()
                        }));
                    if execute_request_btn_resp.clicked() && executos_is_free {
                        request.go(Arc::clone(&states.events), true);
                    } else if execute_request_btn_resp.clicked() && !executos_is_free {
                        request.termiate();
                    };
                });
            });
        });
    }
}
