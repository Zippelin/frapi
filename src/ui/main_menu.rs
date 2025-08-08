use std::{path::PathBuf, process::exit, sync::Arc};

use egui::{
    vec2, Button, Context, CornerRadius, Frame, Label, Margin, MenuBar, Pos2, RichText,
    TopBottomPanel, Visuals, WidgetText, Window,
};
use rfd::FileDialog;

use crate::states::States;

pub struct MainMenu {
    export_folder_path: Option<PathBuf>,
    import_file_path: Option<PathBuf>,
    modal_about_is_visilbe: bool,
}

impl MainMenu {
    pub fn new() -> Self {
        Self {
            export_folder_path: None,
            import_file_path: None,
            modal_about_is_visilbe: false,
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
                MenuBar::new().ui(ui, |ui| {
                    // Паддинг для высокоуровневыхз кнопок меню
                    ui.style_mut().spacing.button_padding = vec2(10., 10.);
                    // Убираем скругление для панели меню
                    ui.ctx().set_visuals(Visuals {
                        menu_corner_radius: CornerRadius::ZERO,
                        window_fill: states.style.color_secondary(),
                        // extreme_bg_color: states.style.color_secondary(),
                        ..ui.visuals().clone()
                    });

                    ui.menu_button(
                        states
                            .style
                            .fonts
                            .menu_text("File")
                            .color(states.style.color_main()),
                        |ui| {
                            ui.style_mut().spacing.button_padding = vec2(10., 10.);
                            ui.style_mut().visuals.widgets.hovered.weak_bg_fill =
                                states.style.color_lighter();
                            ui.style_mut().visuals.widgets.open.weak_bg_fill =
                                states.style.color_lighter();

                            if ui
                                .button(
                                    states
                                        .style
                                        .fonts
                                        .menu_text("Save All               Ctrl+Shift+S")
                                        .color(states.style.color_main()),
                                )
                                .clicked()
                            {
                                states.save(None);
                            };

                            if ui
                                .button(
                                    states
                                        .style
                                        .fonts
                                        .menu_text("Save Selected          Ctrl+S")
                                        .color(states.style.color_main()),
                                )
                                .clicked()
                            {
                                states.save_selected(None);
                            };

                            ui.menu_button(
                                states
                                    .style
                                    .fonts
                                    .menu_text("Export / Import")
                                    .color(states.style.color_main()),
                                |ui| {
                                    ui.style_mut().spacing.button_padding = vec2(10., 10.);
                                    ui.style_mut().visuals.widgets.hovered.weak_bg_fill =
                                        states.style.color_lighter();
                                    ui.style_mut().visuals.widgets.open.weak_bg_fill =
                                        states.style.color_lighter();

                                    if ui
                                        .button(
                                            states
                                                .style
                                                .fonts
                                                .menu_text("Export Settings To...")
                                                .color(states.style.color_main()),
                                        )
                                        .clicked()
                                    {
                                        self.folders_picker();
                                    };

                                    if ui
                                        .button(
                                            states
                                                .style
                                                .fonts
                                                .menu_text("Import Settings From...")
                                                .color(states.style.color_main()),
                                        )
                                        .clicked()
                                    {
                                        self.file_picker();
                                    };
                                },
                            );

                            if self.export_folder_path.is_some() {
                                states.save(self.export_folder_path.take());
                            };

                            if self.import_file_path.is_some() {
                                states.load(self.import_file_path.take());
                            };

                            ui.separator();

                            if ui
                                .button(
                                    states
                                        .style
                                        .fonts
                                        .menu_text("Exit             ")
                                        .color(states.style.color_main()),
                                )
                                .clicked()
                            {
                                exit(0)
                            };
                        },
                    );

                    ui.menu_button(
                        states
                            .style
                            .fonts
                            .menu_text("Options")
                            .color(states.style.color_main()),
                        |ui| {
                            ui.style_mut().spacing.button_padding = vec2(10., 10.);
                            ui.style_mut().visuals.widgets.hovered.weak_bg_fill =
                                states.style.color_lighter();
                            ui.style_mut().visuals.widgets.open.weak_bg_fill =
                                states.style.color_lighter();

                            ui.menu_button(
                                states
                                    .style
                                    .fonts
                                    .menu_text("Settings")
                                    .color(states.style.color_main()),
                                |ui| {
                                    ui.style_mut().spacing.button_padding = vec2(10., 10.);
                                    ui.style_mut().visuals.widgets.hovered.weak_bg_fill =
                                        states.style.color_lighter();
                                    ui.style_mut().visuals.widgets.open.weak_bg_fill =
                                        states.style.color_lighter();

                                    ui.menu_button(
                                        states
                                            .style
                                            .fonts
                                            .menu_text("Theme")
                                            .color(states.style.color_main()),
                                        |ui| {
                                            ui.style_mut().spacing.button_padding = vec2(10., 10.);
                                            ui.style_mut().visuals.widgets.hovered.weak_bg_fill =
                                                states.style.color_lighter();
                                            ui.style_mut().visuals.widgets.open.weak_bg_fill =
                                                states.style.color_lighter();

                                            if ui
                                                .add(
                                                    Button::new(
                                                        states
                                                            .style
                                                            .fonts
                                                            .menu_text("Light")
                                                            .color(states.style.color_main()),
                                                    )
                                                    .selected(!states.style.is_dark_theme()),
                                                )
                                                .clicked()
                                            {
                                                states.style.to_light_theme();
                                            };
                                            if ui
                                                .add(
                                                    Button::new(
                                                        states
                                                            .style
                                                            .fonts
                                                            .menu_text("Dark")
                                                            .color(states.style.color_main()),
                                                    )
                                                    .selected(states.style.is_dark_theme()),
                                                )
                                                .clicked()
                                            {
                                                states.style.to_dark_theme();
                                            };
                                        },
                                    );
                                },
                            );
                        },
                    );

                    ui.menu_button(
                        states
                            .style
                            .fonts
                            .menu_text("About")
                            .color(states.style.color_main()),
                        |ui| {
                            ui.style_mut().spacing.button_padding = vec2(10., 10.);
                            ui.style_mut().visuals.widgets.hovered.weak_bg_fill =
                                states.style.color_lighter();
                            ui.style_mut().visuals.widgets.open.weak_bg_fill =
                                states.style.color_lighter();

                            if ui
                                .button(
                                    states
                                        .style
                                        .fonts
                                        .menu_text("About Frapi   ")
                                        .color(states.style.color_main()),
                                )
                                .clicked()
                            {
                                self.modal_about_is_visilbe = true;
                            }
                        },
                    );
                });
            });
        self.about_window(ctx, states);
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

    fn about_window(&mut self, ctx: &Context, states: &mut States) {
        let window_size = vec2(300., 250.);

        Window::new("About Frapi")
            .collapsible(false)
            .resizable(true)
            // Position is relative to main window inner coords
            .fixed_pos(Pos2::new(
                ctx.screen_rect().center().x - window_size.x / 2.,
                ctx.screen_rect().center().y - window_size.y / 2.,
            ))
            .fixed_size(window_size)
            .open(&mut self.modal_about_is_visilbe)
            .show(ctx, |ui| {
                Frame::default()
                    .fill(states.style.color_main())
                    .inner_margin(Margin::same(10))
                    .corner_radius(CornerRadius::same(5))
                    .show(ui, |ui| {
                        // Hack for Window, since it wont react on fixed_size
                        ui.set_width(ui.available_width());
                        ui.set_height(ui.available_height());

                        ui.add(
                            Label::new(WidgetText::RichText(Arc::new(
                                RichText::new("About Frapi - Free API")
                                    .size(15.)
                                    .monospace(),
                            )))
                            .selectable(false),
                        );

                        ui.add_space(10.);
                        ui.add(Label::new("Ver: 0.1.0").selectable(false));
                        ui.add_space(10.);
                        ui.add(
                            Label::new(
                                "This is free and light HTTP\\WS requests tracker and tester. Easy to use and easy to copy between desktops.",
                            )
                            .selectable(false),
                        );
                        ui.add(
                            Label::new("Application based on Rust lang, powered by Egui.").selectable(false),
                        );
                        ui.add_space(10.);
                        ui.add(Label::new("License: MiT").selectable(false));
                        ui.add(Label::new("GitHub: https://github.com/Zippelin/frapi").selectable(true));
                        ui.add_space(10.);
                        ui.add(Label::new("@2025 Frapi Team").selectable(false));
                    })
            });
    }
}
