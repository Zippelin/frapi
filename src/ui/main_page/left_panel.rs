use egui::{
    text::LayoutJob, vec2, Align, Color32, Context, FontFamily, FontId, Frame, Layout, ScrollArea,
    SidePanel, TextFormat, Ui,
};

use crate::{
    states::{
        main_page::{Entity, Request},
        States,
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
            .width_range(150.0..=300.)
            .frame(Frame::new().fill(states.style.color_main()))
            .resizable(true)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.group(|ui| {
                        ui.menu_button("New", |ui| {
                            let _ = ui.button("Request");
                            let _ = ui.button("Collection");
                        });

                        ui.text_edit_singleline(&mut states.main_page.filter_text)
                    });
                });
                ui.add_space(5.);

                // Список сущностей
                ui.with_layout(Layout::top_down_justified(Align::LEFT), |ui| {
                    ui.style_mut().spacing.scroll.bar_outer_margin = 2.;

                    ScrollArea::vertical()
                        .max_height(ui.available_height() - 10.)
                        .show(ui, |ui| {
                            Frame::default().show(ui, |ui| {
                                ui.style_mut().spacing.button_padding = vec2(0., 5.);

                                for i in 0..states.main_page.entities.len() {
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
        match &mut states.main_page.entities[entity_idx] {
            Entity::COLLECTION(collection) => {
                let is_selected = states
                    .main_page
                    .selected_entity
                    .collection_is_selected(entity_idx);

                let collection_text = if collection.is_changed {
                    &format!("{} *", &collection.original.name)
                } else {
                    &collection.original.name
                };
                let collection_folder_response = ui.selectable_label(
                    is_selected,
                    self.get_collection_text(!collection.is_folded, &collection_text),
                );
                if collection_folder_response.clicked() {
                    states
                        .main_page
                        .selected_entity
                        .select_collection(entity_idx);
                };

                if collection_folder_response.double_clicked() {
                    collection.is_folded = !collection.is_folded;
                }

                if !collection.is_folded {
                    for request_idx in 0..collection.requests.len() {
                        let is_selected = states
                            .main_page
                            .selected_entity
                            .request_is_selected(Some(entity_idx), request_idx);

                        ui.indent(format!("entity-{}-{}", entity_idx, request_idx), |ui| {
                            if ui
                                .selectable_label(
                                    is_selected,
                                    self.get_request_text(
                                        is_selected,
                                        &collection.requests[request_idx],
                                    ),
                                )
                                .clicked()
                            {
                                states
                                    .main_page
                                    .selected_entity
                                    .select_request(Some(entity_idx), request_idx);
                            };
                        });
                    }
                }
            }
            Entity::REQUEST(request) => {
                let is_selected = states
                    .main_page
                    .selected_entity
                    .request_is_selected(None, entity_idx);

                if ui
                    .selectable_label(is_selected, self.get_request_text(is_selected, &request))
                    .clicked()
                {
                    states
                        .main_page
                        .selected_entity
                        .select_request(None, entity_idx);
                };
            }
        };
    }

    /// paint request item text
    fn get_request_text(&self, is_selected: bool, request: &Request) -> LayoutJob {
        let _ = is_selected;
        let mut job = LayoutJob::default();
        job.break_on_newline = true;

        job.append(
            &format!(
                "  {}\n  {}  ",
                &request.original.protocol.to_string().to_uppercase(),
                &request.original.method.to_string().to_uppercase()
            ),
            0.,
            TextFormat {
                color: Color32::LIGHT_GREEN,
                font_id: FontId::new(6.0, FontFamily::Monospace),

                ..Default::default()
            },
        );

        let reqeust_text = if request.is_changed {
            &format!("{} *", &request.original.name)
        } else {
            &request.original.name
        };
        job.append(
            &reqeust_text,
            5.,
            TextFormat {
                color: Color32::LIGHT_GRAY,
                font_id: FontId::new(13.0, FontFamily::Proportional),
                ..Default::default()
            },
        );
        job
    }

    /// paint collection item text
    fn get_collection_text(&self, is_selected: bool, text: &String) -> LayoutJob {
        let mut job = LayoutJob::default();

        let icon = if is_selected {
            &Icon::folder_open()
        } else {
            &Icon::folder_closed()
        };
        job.append(
            icon,
            10.,
            TextFormat {
                color: Color32::LIGHT_GRAY,
                font_id: FontId::new(15.0, FontFamily::Proportional),
                ..Default::default()
            },
        );
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
