use std::sync::Arc;

use egui::{
    text::LayoutJob, vec2, Align, CentralPanel, Color32, FontFamily, FontId, Frame, Grid, Label,
    Layout, Margin, RichText, ScrollArea, TextEdit, TextFormat, TextWrapMode, Ui, WidgetText,
};

use crate::{
    settings::Protocol,
    states::{
        main_page::response::{Response, ResponseView},
        States, Style,
    },
    ui::{icons::Icon, main_page::central_panel::responses::json_view::JsonView},
};

mod json_view;

pub struct ResponsesListPanel {
    json_view: JsonView,
}

impl ResponsesListPanel {
    pub fn new() -> Self {
        Self {
            json_view: JsonView::new(),
        }
    }

    pub fn update(&self, ui: &mut Ui, states: &mut States) {
        CentralPanel::default()
            .frame(
                Frame::new()
                    .fill(states.style.color_main())
                    .inner_margin(Margin::same(10)),
            )
            .show_inside(ui, |ui| {
                let request = states.main_page.selected_request_mut();

                if request.is_none() {
                    return;
                }

                let request = request.unwrap();
                let responses = request.responses.lock();

                let protocol = &request.draft.protocol;
                if responses.is_err() {
                    return;
                }

                let mut responses = responses.unwrap();

                // TODO: add clear all responses
                ui.with_layout(Layout::top_down_justified(Align::LEFT), |ui| {
                    ScrollArea::vertical()
                        .max_height(ui.available_height() - 10.)
                        .show(ui, |ui| {
                            Frame::new().inner_margin(Margin::same(10)).show(ui, |ui| {
                                for i in (0..responses.len()).rev() {
                                    self.update_response(
                                        ui,
                                        &mut responses[i],
                                        protocol,
                                        &states.style,
                                    );
                                    ui.separator();
                                }
                            });
                        });
                });
            });
    }

    fn update_response(
        &self,
        ui: &mut Ui,
        response: &mut Response,
        protocol: &Protocol,
        style: &Style,
    ) {
        ui.style_mut().spacing.button_padding = vec2(10., 10.);
        if ui
            .selectable_label(
                !response.is_folded,
                self.get_response_header(!response.is_folded, protocol, &response),
            )
            .clicked()
        {
            response.is_folded = !response.is_folded;
        };

        if !response.is_folded {
            self.update_response_unfolded(ui, response, style);
        }
    }

    fn update_response_unfolded(&self, ui: &mut Ui, response: &mut Response, style: &Style) {
        ui.horizontal(|ui| {
            ui.radio_value(&mut response.selected_view, ResponseView::RAW, "Raw");
            if response.data.json_is_exist() {
                ui.radio_value(&mut response.selected_view, ResponseView::JSON, "Json");
            }

            ui.radio_value(
                &mut response.selected_view,
                ResponseView::HEADERS,
                "Headers",
            );
        });
        match response.selected_view {
            ResponseView::JSON => {
                self.json_view.update(ui, response, style);
            }
            ResponseView::RAW => {
                let mut raw_text = response.data.raw.clone();
                ui.add(
                    TextEdit::multiline(&mut raw_text)
                        .code_editor()
                        .text_color(style.color_main())
                        .background_color(style.color_light()),
                );
            }
            ResponseView::HEADERS => {
                Frame::new()
                    .inner_margin(Margin::same(5))
                    .fill(style.color_main())
                    .show(ui, |ui| {
                        Grid::new(format!("response-headers-{}", response.time))
                            .num_columns(2)
                            .spacing(vec2(2., 5.))
                            .min_col_width(100.)
                            .striped(true)
                            .show(ui, |ui| {
                                if response.data.redictection_url.len() > 0 {
                                    ui.add(
                                        Label::new("Redirect URL").wrap_mode(TextWrapMode::Wrap),
                                    );

                                    ui.add(
                                        Label::new(WidgetText::RichText(Arc::new(
                                            RichText::new(response.data.redictection_url.clone())
                                                .font(style.fonts.label_strong())
                                                .strong(),
                                        )))
                                        .wrap_mode(TextWrapMode::Wrap),
                                    );

                                    ui.end_row();
                                }

                                for header in &response.data.headers {
                                    ui.add(
                                        Label::new(header.key.clone())
                                            .wrap_mode(TextWrapMode::Wrap),
                                    );

                                    ui.add(
                                        Label::new(WidgetText::RichText(Arc::new(
                                            RichText::new(header.value.clone())
                                                .font(style.fonts.label_strong())
                                                .strong(),
                                        )))
                                        .wrap_mode(TextWrapMode::Wrap),
                                    );

                                    ui.end_row();
                                }
                            })
                    });
            }
        };
    }

    fn get_response_header(
        &self,
        is_selected: bool,
        protocol: &Protocol,
        response: &Response,
    ) -> LayoutJob {
        let mut job = LayoutJob::default();
        let icon = if is_selected {
            &Icon::triangle_bottom()
        } else {
            &Icon::triangle_right()
        };

        job.append(
            icon,
            10.,
            TextFormat {
                color: Color32::LIGHT_GRAY,
                font_id: FontId::new(15.0, FontFamily::Monospace),
                ..Default::default()
            },
        );
        job.append(
            &format!("{}", response.time.format("%Y-%m-%d %H:%M:%S")),
            5.,
            TextFormat {
                color: Color32::LIGHT_GRAY,
                font_id: FontId::new(13.0, FontFamily::Proportional),
                ..Default::default()
            },
        );

        if [Protocol::HTTP, Protocol::HTTPS].contains(protocol) {
            job.append(
                "       ",
                0.0,
                TextFormat {
                    color: Color32::LIGHT_BLUE,
                    font_id: FontId::new(12.0, FontFamily::Proportional),
                    ..Default::default()
                },
            );

            let (code_color, code_text) = if 100 <= response.code && response.code <= 199 {
                (Color32::BLUE, format!("{} INFO", response.code))
            } else if 200 <= response.code && response.code <= 200 {
                (Color32::GREEN, format!("{} OK", response.code))
            } else if 300 <= response.code && response.code <= 399 {
                (Color32::YELLOW, format!("{} REDIRECT", response.code))
            } else if 400 <= response.code && response.code <= 499 {
                (Color32::RED, format!("{} CLIENT ERROR", response.code))
            } else {
                (Color32::ORANGE, format!("{} SERVER ERROR", response.code))
            };

            job.append(
                &code_text,
                0.0,
                TextFormat {
                    color: code_color,
                    font_id: FontId::new(14.0, FontFamily::Monospace),
                    ..Default::default()
                },
            );
        }

        job
    }
}
