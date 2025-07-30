use std::sync::Arc;

use egui::{
    text::LayoutJob, vec2, Align, Button, CollapsingHeader, Color32, FontFamily, FontId, Frame,
    Label, Layout, RichText, ScrollArea, TextEdit, TextFormat, Ui, WidgetText,
};
use serde_json::Value;

use crate::states::{
    main_page::response::{JsonViewType, Response},
    Style,
};

pub struct JsonView {}

impl JsonView {
    pub fn new() -> Self {
        Self {}
    }

    pub fn update(&self, ui: &mut Ui, response: &mut Response, style: &Style) {
        ui.group(|ui| {
            ui.with_layout(Layout::top_down_justified(Align::RIGHT), |ui| {
                ui.horizontal(|ui| {
                    ui.style_mut().spacing.button_padding = vec2(5., 3.);
                    let simple_view_btn = Button::new("Simple")
                        .selected(response.data.json.view_type == JsonViewType::Simple);
                    let complex_view_btn = Button::new("Complex")
                        .selected(response.data.json.view_type == JsonViewType::Comlex);

                    if ui.add(complex_view_btn).clicked() {
                        response.data.json.view_type = JsonViewType::Comlex
                    }
                    if ui.add(simple_view_btn).clicked() {
                        response.data.json.view_type = JsonViewType::Simple
                    };
                })
            })
        });

        if response.data.json.view_type == JsonViewType::Comlex {
            self.update_complex(ui, response, style);
        }

        if response.data.json.view_type == JsonViewType::Simple {
            self.update_simple(ui, response, style);
        }
    }

    fn update_simple(&self, ui: &mut Ui, response: &Response, style: &Style) {
        ui.with_layout(Layout::top_down_justified(Align::LEFT), |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                let mut text = response.data.json.simple.clone();
                let te = TextEdit::multiline(&mut text)
                    .code_editor()
                    .text_color(style.color_main())
                    .desired_rows(20);
                ui.add(te);
            })
        });
    }

    fn update_complex(&self, ui: &mut Ui, response: &Response, style: &Style) {
        ui.with_layout(Layout::top_down_justified(Align::LEFT), |ui| {
            if !response.data.json_is_exist() {
                ui.label("{}");
                return;
            }

            ScrollArea::vertical()
                .scroll([true, true])
                .max_height(300.)
                .show(ui, |ui| {
                    Frame::default()
                        .fill(style.color_secondary())
                        .show(ui, |ui| {
                            // Используется как пустой сепаратор
                            ui.add_sized(vec2(ui.available_width(), 1.), Label::new(""));
                            //
                            match &response.data.json.complex {
                                Value::Object(map) => {
                                    CollapsingHeader::new(WidgetText::RichText(Arc::new(
                                        RichText::new("[root object]")
                                            .color(Color32::LIGHT_GRAY)
                                            .size(11.),
                                    )))
                                    .default_open(true)
                                    .id_salt(format!("{:#?}", map))
                                    .show(ui, |ui| {
                                        for (map_key, map_value) in map {
                                            match map_value {
                                                Value::Null => {
                                                    ui.horizontal(|ui| {
                                                        ui.add(self.label_for_key(&map_key));
                                                        ui.add(self.label_for_straight_value(
                                                            &"null".to_string(),
                                                        ));
                                                    });
                                                    continue;
                                                }
                                                Value::Bool(val) => {
                                                    let label = self.label_for_key(&map_key);
                                                    ui.horizontal(|ui| {
                                                        ui.add(label);
                                                        ui.add(self.label_for_straight_value(
                                                            &val.to_string(),
                                                        ));
                                                    });
                                                    continue;
                                                }
                                                Value::Number(number) => {
                                                    ui.horizontal(|ui| {
                                                        ui.add(self.label_for_key(&map_key));
                                                        ui.add(self.label_for_straight_value(
                                                            &number.to_string(),
                                                        ));
                                                    });
                                                    continue;
                                                }
                                                Value::String(val) => {
                                                    ui.horizontal(|ui| {
                                                        ui.add(self.label_for_key(&map_key));
                                                        ui.add(self.label_for_text_value(&val));
                                                    });
                                                    continue;
                                                }
                                                Value::Array(_) => {
                                                    ui.with_layout(
                                                        Layout::top_down_justified(Align::LEFT),
                                                        |ui| {
                                                            CollapsingHeader::new(
                                                                self.text_for_vec_key(&map_key),
                                                            )
                                                            .default_open(true)
                                                            .id_salt(format!(
                                                                "{:#?}-{:#?}",
                                                                map_key, map_value
                                                            ))
                                                            .show(ui, |ui| {
                                                                self.update_fold_tree(
                                                                    ui, &map_value,
                                                                );
                                                            });
                                                        },
                                                    );
                                                }
                                                Value::Object(_) => {
                                                    ui.with_layout(
                                                        Layout::top_down(Align::LEFT),
                                                        |ui| {
                                                            CollapsingHeader::new(
                                                                self.text_for_obj_key(&map_key),
                                                            )
                                                            .default_open(true)
                                                            .id_salt(format!(
                                                                "{:#?}-{:#?}",
                                                                map_key, map_value
                                                            ))
                                                            .show(ui, |ui| {
                                                                self.update_fold_tree(
                                                                    ui, &map_value,
                                                                );
                                                            });
                                                        },
                                                    );
                                                }
                                            }
                                        }
                                    });
                                }
                                _ => {
                                    return;
                                }
                            };
                        });
                });
        });
    }

    fn update_fold_tree(&self, ui: &mut Ui, data: &Value) {
        match data {
            Value::Array(items) => {
                for i in 0..items.len() {
                    match &items[i] {
                        Value::Null => {
                            ui.horizontal(|ui| {
                                ui.add(self.label_for_index(&i));
                                ui.add(self.label_for_straight_value(&"null".to_string()));
                            });
                            continue;
                        }
                        Value::Bool(val) => {
                            ui.horizontal(|ui| {
                                ui.add(self.label_for_index(&i));
                                ui.add(self.label_for_straight_value(&val.to_string()));
                            });
                            continue;
                        }
                        Value::Number(number) => {
                            ui.horizontal(|ui| {
                                ui.add(self.label_for_index(&i));
                                ui.add(self.label_for_straight_value(&number.to_string()));
                            });
                            continue;
                        }
                        Value::String(val) => {
                            ui.horizontal(|ui| {
                                ui.add(self.label_for_index(&i));
                                ui.add(self.label_for_text_value(&val.to_string()));
                            });
                            continue;
                        }
                        Value::Array(inner_items) => {
                            ui.horizontal(|ui| {
                                // ui.add(self.label_for_index(&i));
                                ui.with_layout(Layout::top_down(Align::LEFT), |ui| {
                                    CollapsingHeader::new(self.text_for_index_as_vec(&i))
                                        .default_open(true)
                                        .id_salt(format!("{:#?}", items[i]))
                                        .show(ui, |ui| {
                                            for j in 0..inner_items.len() {
                                                self.update_fold_tree(ui, &inner_items[j]);
                                            }
                                        });
                                });
                            });
                        }
                        Value::Object(_) => {
                            ui.horizontal(|ui| {
                                // ui.add(self.label_for_index(&i));
                                ui.with_layout(Layout::top_down(Align::LEFT), |ui| {
                                    CollapsingHeader::new(self.text_for_index_as_object(&i))
                                        .default_open(true)
                                        .id_salt(format!("{:#?}", items[i]))
                                        .show(ui, |ui| {
                                            self.update_fold_tree(ui, &items[i]);
                                        });
                                });
                            });
                        }
                    }
                }
            }
            Value::Object(fields) => {
                for (field_key, field_value) in fields {
                    match field_value {
                        Value::Null => {
                            ui.horizontal(|ui| {
                                ui.add(self.label_for_key(&field_key));
                                ui.add(self.label_for_straight_value(&"null".to_string()));
                            });
                            continue;
                        }
                        Value::Bool(val) => {
                            ui.horizontal(|ui| {
                                ui.add(self.label_for_key(&field_key));
                                ui.add(self.label_for_straight_value(&val.to_string()));
                            });
                            continue;
                        }
                        Value::Number(number) => {
                            ui.horizontal(|ui| {
                                ui.add(self.label_for_key(&field_key));
                                ui.add(self.label_for_straight_value(&number.to_string()));
                            });
                            continue;
                        }
                        Value::String(val) => {
                            ui.horizontal(|ui| {
                                ui.add(self.label_for_key(&field_key));
                                ui.add(self.label_for_text_value(val));
                            });
                            continue;
                        }
                        Value::Array(inner_items) => {
                            ui.with_layout(Layout::top_down(Align::LEFT), |ui| {
                                CollapsingHeader::new(self.text_for_obj_key(&field_key))
                                    .default_open(true)
                                    .id_salt(format!("{:#?}-{:#?}", &field_key, &field_value))
                                    .show(ui, |ui| {
                                        for j in 0..inner_items.len() {
                                            self.update_fold_tree(ui, &inner_items[j]);
                                        }
                                    });
                            });
                        }
                        Value::Object(_) => {
                            ui.with_layout(Layout::top_down(Align::LEFT), |ui| {
                                CollapsingHeader::new(self.text_for_obj_key(&field_key))
                                    .default_open(true)
                                    .id_salt(format!("{:#?}-{:#?}", &field_key, &field_value))
                                    .show(ui, |ui| {
                                        self.update_fold_tree(ui, &field_value);
                                    });
                            });
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn label_for_text_value(&self, text: &String) -> Label {
        let mut job = LayoutJob::default();
        job.append(
            "\"",
            0.0,
            TextFormat {
                color: Color32::LIGHT_BLUE,
                font_id: FontId::new(13.0, FontFamily::Proportional),
                ..Default::default()
            },
        );
        job.append(
            &text,
            0.0,
            TextFormat {
                color: Color32::LIGHT_GREEN,
                font_id: FontId::new(13.0, FontFamily::Proportional),
                ..Default::default()
            },
        );
        job.append(
            "\"",
            0.0,
            TextFormat {
                color: Color32::LIGHT_BLUE,
                font_id: FontId::new(13.0, FontFamily::Proportional),
                ..Default::default()
            },
        );
        job.append(
            ",",
            0.0,
            TextFormat {
                color: Color32::LIGHT_YELLOW,
                font_id: FontId::new(13.0, FontFamily::Proportional),
                ..Default::default()
            },
        );
        Label::new(WidgetText::LayoutJob(Arc::new(job)))
    }

    fn label_for_straight_value(&self, text: &String) -> Label {
        let mut job = LayoutJob::default();

        job.append(
            &text.clone(),
            0.0,
            TextFormat {
                color: Color32::LIGHT_BLUE,
                font_id: FontId::new(13.0, FontFamily::Proportional),
                ..Default::default()
            },
        );
        job.append(
            ",",
            0.0,
            TextFormat {
                color: Color32::LIGHT_YELLOW,
                font_id: FontId::new(13.0, FontFamily::Proportional),
                ..Default::default()
            },
        );

        Label::new(WidgetText::LayoutJob(Arc::new(job)))
    }

    fn label_for_index(&self, index: &usize) -> Label {
        let mut job = self.text_for_index(index);

        job.append(
            ":",
            0.0,
            TextFormat {
                color: Color32::LIGHT_YELLOW,
                font_id: FontId::new(11.0, FontFamily::Proportional),
                ..Default::default()
            },
        );
        Label::new(WidgetText::LayoutJob(Arc::new(job)))
    }

    fn text_for_index(&self, index: &usize) -> LayoutJob {
        let mut job = LayoutJob::default();

        job.append(
            &index.to_string(),
            0.0,
            TextFormat {
                color: Color32::DARK_GRAY,
                font_id: FontId::new(11.0, FontFamily::Monospace),
                ..Default::default()
            },
        );

        job
    }

    fn text_for_index_as_object(&self, index: &usize) -> WidgetText {
        let mut job = self.text_for_index(index);

        job.append(
            ":",
            0.0,
            TextFormat {
                color: Color32::LIGHT_GREEN,
                font_id: FontId::new(12.0, FontFamily::Proportional),
                ..Default::default()
            },
        );

        job.append(
            "[object]",
            15.0,
            TextFormat {
                color: Color32::LIGHT_GRAY,
                font_id: FontId::new(11.0, FontFamily::Proportional),
                ..Default::default()
            },
        );
        WidgetText::LayoutJob(Arc::new(job))
    }

    fn text_for_index_as_vec(&self, index: &usize) -> WidgetText {
        let mut job = self.text_for_index(index);

        job.append(
            ":",
            0.0,
            TextFormat {
                color: Color32::LIGHT_GREEN,
                font_id: FontId::new(12.0, FontFamily::Proportional),
                ..Default::default()
            },
        );

        job.append(
            "[array]",
            15.0,
            TextFormat {
                color: Color32::LIGHT_GRAY,
                font_id: FontId::new(11.0, FontFamily::Proportional),
                ..Default::default()
            },
        );

        WidgetText::LayoutJob(Arc::new(job))
    }

    fn label_for_key(&self, text: &String) -> Label {
        let mut job = LayoutJob::default();
        job.append(
            "\"",
            0.0,
            TextFormat {
                color: Color32::LIGHT_BLUE,
                font_id: FontId::new(12.0, FontFamily::Proportional),
                ..Default::default()
            },
        );
        job.append(
            &text.clone(),
            0.0,
            TextFormat {
                color: Color32::YELLOW,
                font_id: FontId::new(12.0, FontFamily::Proportional),
                ..Default::default()
            },
        );
        job.append(
            "\"",
            0.0,
            TextFormat {
                color: Color32::LIGHT_BLUE,
                font_id: FontId::new(12.0, FontFamily::Proportional),
                ..Default::default()
            },
        );
        job.append(
            ":",
            0.0,
            TextFormat {
                color: Color32::LIGHT_GREEN,
                font_id: FontId::new(12.0, FontFamily::Proportional),
                ..Default::default()
            },
        );
        Label::new(WidgetText::LayoutJob(Arc::new(job)))
    }

    fn text_for_obj_key(&self, text: &String) -> WidgetText {
        let mut job = LayoutJob::default();

        if text.len() > 0 {
            job.append(
                "\"",
                0.0,
                TextFormat {
                    color: Color32::LIGHT_BLUE,
                    font_id: FontId::new(12.0, FontFamily::Proportional),
                    ..Default::default()
                },
            );
            job.append(
                &text,
                0.0,
                TextFormat {
                    color: Color32::ORANGE,
                    font_id: FontId::new(12.0, FontFamily::Proportional),
                    ..Default::default()
                },
            );
            job.append(
                "\"",
                0.0,
                TextFormat {
                    color: Color32::LIGHT_BLUE,
                    font_id: FontId::new(12.0, FontFamily::Proportional),
                    ..Default::default()
                },
            );
            job.append(
                ":",
                0.0,
                TextFormat {
                    color: Color32::LIGHT_GREEN,
                    font_id: FontId::new(12.0, FontFamily::Proportional),
                    ..Default::default()
                },
            );
        };

        job.append(
            "[object]",
            15.0,
            TextFormat {
                color: Color32::LIGHT_GRAY,
                font_id: FontId::new(11.0, FontFamily::Proportional),
                ..Default::default()
            },
        );

        WidgetText::LayoutJob(Arc::new(job))
    }

    fn text_for_vec_key(&self, text: &String) -> WidgetText {
        let mut job = LayoutJob::default();
        job.append(
            "\"",
            0.0,
            TextFormat {
                color: Color32::LIGHT_BLUE,
                font_id: FontId::new(12.0, FontFamily::Proportional),
                ..Default::default()
            },
        );
        job.append(
            &text,
            0.0,
            TextFormat {
                color: Color32::ORANGE,
                font_id: FontId::new(12.0, FontFamily::Proportional),
                ..Default::default()
            },
        );
        job.append(
            "\"",
            0.0,
            TextFormat {
                color: Color32::LIGHT_BLUE,
                font_id: FontId::new(12.0, FontFamily::Proportional),
                ..Default::default()
            },
        );
        job.append(
            ":",
            0.0,
            TextFormat {
                color: Color32::LIGHT_GREEN,
                font_id: FontId::new(12.0, FontFamily::Proportional),
                ..Default::default()
            },
        );

        job.append(
            "[array]",
            15.0,
            TextFormat {
                color: Color32::LIGHT_GRAY,
                font_id: FontId::new(11.0, FontFamily::Proportional),
                ..Default::default()
            },
        );
        WidgetText::LayoutJob(Arc::new(job))
    }
}
