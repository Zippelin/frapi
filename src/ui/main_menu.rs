use std::{path::PathBuf, process::exit};

use egui::{
    menu::{self},
    vec2, Context, CornerRadius, Frame, TopBottomPanel, Visuals,
};
use rfd::FileDialog;

use crate::states::States;

pub struct MainMenu {
    export_folder_path: Option<PathBuf>,
    import_file_path: Option<PathBuf>,
}

impl MainMenu {
    pub fn new() -> Self {
        Self {
            export_folder_path: None,
            import_file_path: None,
        }
    }

    pub fn update(&mut self, ctx: &Context, states: &mut States) {
        TopBottomPanel::top("main_menubar")
            .frame(
                Frame::new()
                    .corner_radius(CornerRadius::ZERO)
                    .fill(states.style.color_secondary()),
            )
            .show(ctx, |ui| {
                ui.style_mut().visuals.menu_corner_radius = CornerRadius::ZERO;
                menu::bar(ui, |ui| {
                    // Паддинг для высокоуровневыхз кнопок меню
                    ui.style_mut().spacing.button_padding = vec2(10., 10.);
                    // Убираем скругление для панели меню
                    ui.ctx().set_visuals(Visuals {
                        menu_corner_radius: CornerRadius::ZERO,
                        window_fill: states.style.color_secondary(),
                        extreme_bg_color: states.style.color_light(),
                        ..ui.visuals().clone()
                    });

                    ui.menu_button("File", |ui| {
                        // TODO: add Import settings

                        if ui.button("Save             ").clicked() {
                            states.save(None);
                        };

                        ui.menu_button("Export / Import", |ui| {
                            if ui.button("Export Settings To...").clicked() {
                                ui.close_menu();
                                self.folders_picker();
                            };

                            if ui.button("Import Settings From...").clicked() {
                                ui.close_menu();
                                self.file_picker();
                            };
                        });

                        if self.export_folder_path.is_some() {
                            states.save(self.export_folder_path.take());
                        };

                        if self.import_file_path.is_some() {
                            states.load(self.import_file_path.take());
                        };

                        ui.separator();

                        if ui.button("Exit             ").clicked() {
                            exit(0)
                        };
                    });

                    ui.menu_button("Options", |ui| {
                        let _ = ui.button("Settings             ");
                    });

                    ui.menu_button("About", |ui| {
                        let _ = ui.button("About Frapi             ");
                    });
                });
            });
    }

    fn folders_picker(&mut self) {
        let folder = FileDialog::new()
            .set_directory("./")
            .set_file_name("settings.json")
            .add_filter("json", &["json"])
            .save_file();
        self.export_folder_path = folder;
    }

    fn file_picker(&mut self) {
        let file = FileDialog::new()
            .set_directory("./")
            .set_file_name("settings.json")
            .add_filter("json", &["json"])
            .pick_file();
        self.import_file_path = file;
    }
}
