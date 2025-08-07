use egui::{Context, Frame, Label, Margin, RichText, Separator, TopBottomPanel};

use crate::states::States;

pub struct BottomPanel {}

impl BottomPanel {
    pub fn new() -> Self {
        Self {}
    }

    pub fn update(&mut self, ctx: &Context, states: &mut States) {
        TopBottomPanel::bottom("main-bottom-panel")
            .frame(
                Frame::new()
                    .fill(states.style.color_secondary())
                    .inner_margin(Margin::same(3)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(ui.available_width() - 400.);

                    ui.add(Label::new(RichText::new("Items: ").color(states.style.color_main())).selectable(false));
                    ui.add(
                        Label::new(RichText::new(format!("{}", states.main_page.get_selected_items_count())).color(states.style.color_main()))
                            .selectable(false),
                    )
                    .on_hover_text("Amount of items on root level or in collection");

                    ui.add_space(10.);
                    ui.add(Separator::default().vertical());
                    ui.add_space(10.);

                    ui.add(Label::new(RichText::new("Executor: ").color(states.style.color_main())).selectable(false));
                    ui.add(
                        Label::new(RichText::new(states.main_page.selected_request_executor_state()).color(states.style.color_main()))
                            .selectable(false),
                    )
                    .on_hover_text(
                        "Current state of selected Request Executor.\r\nFree - no Request currently running\nBusy - currently http Request executing\nConnected - currently WS connection working",
                    );

                    ui.add_space(10.);
                    ui.add(Separator::default().vertical());
                    ui.add_space(10.);

                    ui.add(Label::new(RichText::new("OS: ").color(states.style.color_main())).selectable(false));

                    let os = match ctx.os() {
                        egui::os::OperatingSystem::Unknown => "Unknown".to_string(),
                        egui::os::OperatingSystem::Android => "Android".to_string(),
                        egui::os::OperatingSystem::IOS => "IOS".to_string(),
                        egui::os::OperatingSystem::Nix => "Nix".to_string(),
                        egui::os::OperatingSystem::Mac => "Mac".to_string(),
                        egui::os::OperatingSystem::Windows => "Windows".to_string(),
                    };
                    ui.add(Label::new(RichText::new(os).color(states.style.color_main())).selectable(false));

                    ui.add_space(10.);
                    ui.add(Separator::default().vertical());
                    ui.add_space(10.);
                })
            });
    }
}
