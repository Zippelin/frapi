use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    vec::IntoIter,
};

use chrono::{DateTime, Local};
use egui::{vec2, Color32, Pos2, Vec2};

use crate::{
    settings::{
        options_settings::OptionsSettings,
        ui_settings::{UISettings, UITheme},
        Settings,
    },
    states::main_page::{entity::Entity, MainPage},
    ui::{colors::ThemeColors, fonts::Fonts},
};

pub mod main_page;

#[derive(Clone, Debug)]
pub struct States {
    pub current_page: Page,
    pub main_page: MainPage,
    pub style: Style,
    pub options: Options,
    pub events: Arc<Mutex<Events>>,
}

impl From<&Settings> for States {
    fn from(value: &Settings) -> Self {
        Self {
            current_page: Page::MAIN,
            main_page: MainPage::from(value),
            style: Style::from(&value.ui),
            options: Options::from(&value.options),
            events: Arc::new(Mutex::new(Events::new())),
        }
    }
}

impl States {
    pub fn event_info(&mut self, msg: &String) {
        self.events.lock().unwrap().event_info(msg);
    }

    pub fn event_warning(&mut self, msg: &String) {
        self.events.lock().unwrap().event_warning(msg);
    }

    pub fn event_error(&mut self, msg: &String) {
        self.events.lock().unwrap().event_error(msg);
    }

    pub fn clear_events(&mut self) {
        self.events.lock().unwrap().clear_events();
    }

    /// Process Entities after save - removing chage status
    pub fn on_save_complete(&mut self) {
        for i in 0..self.main_page.entities.len() {
            match &mut self.main_page.entities[i] {
                Entity::COLLECTION(collection) => collection.on_save(),
                Entity::REQUEST(request) => request.on_save(),
            }
        }
    }

    /// Load settings from path.
    /// Replace partialy currrent states
    pub fn load(&mut self, file_path: Option<PathBuf>) {
        let settings = match Settings::dyn_load(file_path) {
            Ok(val) => val,
            Err(err) => {
                self.event_error(&err);
                return;
            }
        };

        let new_states: Self = Self::from(&settings);
        self.style = new_states.style;
        self.options = new_states.options;
        self.main_page = new_states.main_page;
    }

    /// Saving all data and mark changed entities to unchanged
    pub fn save(&mut self, save_to_path: Option<PathBuf>) {
        match Settings::from(&*self).save(save_to_path) {
            Ok(_) => {
                self.event_info(&"Settings [all] save complete successful".into());
                self.on_save_complete();
            }
            Err(err) => self.event_error(&err),
        };
    }

    /// Update all planned changes for States Data, for next Frame
    pub fn update(&mut self) {
        let move_done = self.main_page.update_request_move();

        let dnd_done = self.main_page.update_dnd();

        let deletion_done = self.main_page.delete_marked_entity();

        if move_done {
            self.save_original(None);
        }
    }

    /// Saving data with original states. Mostly need for savings on moved entity
    pub fn save_original(&mut self, save_to_path: Option<PathBuf>) {
        match Settings::from_original(&*self).save(save_to_path) {
            Ok(_) => {
                self.event_info(&"Settings [original] save complete successful".into());
                self.on_save_complete();
            }
            Err(err) => self.event_error(&err),
        };
    }

    /// Saving only selected Entity
    pub fn save_selected(&mut self, save_to_path: Option<PathBuf>) {
        let request = self.main_page.selected_request_mut();

        if request.is_some() {
            let request = request.unwrap();

            let backup_original_data = request.original.clone();
            request.on_save();

            let save_result = Settings::from_original(&*self).save(save_to_path);

            if save_result.is_err() {
                let err = save_result.unwrap_err();
                let request = self.main_page.selected_request_mut().unwrap();
                request.original.copy_from_other(&backup_original_data);
                request.is_changed = true;
                self.event_error(&format!("Could not save selected request. Error: {err}"))
            } else {
                self.event_info(&"Settings [selected] save complete successful".into());
            }
            return;
        };

        let collection = self.main_page.selected_collection_mut();

        if collection.is_some() {
            let collection = collection.unwrap();

            let backup_original_data = collection.original.clone();
            collection.on_save();

            let save_result = Settings::from_original(&*self).save(save_to_path);

            if save_result.is_err() {
                let err = save_result.unwrap_err();
                let collection = self.main_page.selected_collection_mut().unwrap();
                collection.original.copy_from_other(&backup_original_data);
                collection.is_changed = true;
                self.event_error(&format!("Could not save selected collection. Error: {err}"))
            } else {
                self.event_info(
                    &"Settings save complete successful for selected collection".into(),
                );
            }
            return;
        };
    }

    pub fn events_count(&self) -> usize {
        self.events.lock().unwrap().len()
    }
}

#[derive(Debug, Clone)]
pub enum Theme {
    Light(ThemeColors),
    Dark(ThemeColors),
}

#[derive(Debug, Clone)]
pub struct Style {
    pub theme: Theme,
    pub fonts: Fonts,
}

impl From<&UISettings> for Style {
    fn from(value: &UISettings) -> Self {
        let theme = match value.theme {
            UITheme::Light => Theme::Light(ThemeColors::light()),
            UITheme::Dark => Theme::Dark(ThemeColors::dark()),
        };
        Self {
            theme,
            fonts: Fonts::new(),
        }
    }
}

impl Style {
    pub fn to_dark_theme(&mut self) {
        self.theme = Theme::Dark(ThemeColors::dark())
    }

    pub fn to_light_theme(&mut self) {
        self.theme = Theme::Light(ThemeColors::light())
    }

    pub fn color_main(&self) -> Color32 {
        match &self.theme {
            Theme::Light(theme_colors) => theme_colors.main,
            Theme::Dark(theme_colors) => theme_colors.main,
        }
    }

    pub fn color_secondary(&self) -> Color32 {
        match &self.theme {
            Theme::Light(theme_colors) => theme_colors.secondary,
            Theme::Dark(theme_colors) => theme_colors.secondary,
        }
    }

    pub fn color_danger(&self) -> Color32 {
        match &self.theme {
            Theme::Light(theme_colors) => theme_colors.danger,
            Theme::Dark(theme_colors) => theme_colors.danger,
        }
    }

    pub fn color_light(&self) -> Color32 {
        match &self.theme {
            Theme::Light(theme_colors) => theme_colors.light,
            Theme::Dark(theme_colors) => theme_colors.light,
        }
    }

    pub fn color_lighter(&self) -> Color32 {
        match &self.theme {
            Theme::Light(theme_colors) => theme_colors.lighter,
            Theme::Dark(theme_colors) => theme_colors.lighter,
        }
    }

    pub fn color_success(&self) -> Color32 {
        match &self.theme {
            Theme::Light(theme_colors) => theme_colors.success,
            Theme::Dark(theme_colors) => theme_colors.success,
        }
    }

    pub fn color_warning(&self) -> Color32 {
        match &self.theme {
            Theme::Light(theme_colors) => theme_colors.warning,
            Theme::Dark(theme_colors) => theme_colors.warning,
        }
    }
    pub fn is_dark_theme(&self) -> bool {
        match self.theme {
            Theme::Light(_) => false,
            Theme::Dark(_) => true,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Page {
    MAIN,
}

#[derive(Clone, Debug)]
pub struct Options {
    pub window_size: Vec2,
    pub window_position: Option<Pos2>,
}

impl From<&OptionsSettings> for Options {
    fn from(value: &OptionsSettings) -> Self {
        let window_position = if let Some(pos) = value.window_position {
            Some(Pos2 { x: pos.0, y: pos.1 })
        } else {
            None
        };
        Self {
            window_size: vec2(value.window_size.0, value.window_size.1),
            window_position,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Events {
    items: Vec<Event>,
}

impl Events {
    pub fn new() -> Self {
        Self { items: vec![] }
    }
    pub fn event_info(&mut self, msg: &String) {
        self.items.push(Event::info(msg));
    }

    pub fn event_warning(&mut self, msg: &String) {
        self.items.push(Event::warning(msg));
    }

    pub fn event_error(&mut self, msg: &String) {
        self.items.push(Event::error(msg));
    }

    pub fn clear_events(&mut self) {
        self.items.clear();
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn get(&self, i: usize) -> &Event {
        &self.items[i]
    }
}

impl IntoIterator for Events {
    type Item = Event;

    type IntoIter = IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

/// Events across application
#[derive(Debug, Clone)]
pub enum Event {
    Info(EventData),
    Warning(EventData),
    Error(EventData),
}

impl Event {
    pub fn info(msg: &String) -> Self {
        Self::Info(EventData {
            message: msg.clone(),
            timestamp: Local::now(),
        })
    }
    pub fn warning(msg: &String) -> Self {
        Self::Warning(EventData {
            message: msg.clone(),
            timestamp: Local::now(),
        })
    }
    pub fn error(msg: &String) -> Self {
        Self::Error(EventData {
            message: msg.clone(),
            timestamp: Local::now(),
        })
    }

    pub fn get_data(&self) -> &EventData {
        match self {
            Event::Info(event_data) => event_data,
            Event::Warning(event_data) => event_data,
            Event::Error(event_data) => event_data,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EventData {
    pub message: String,
    pub timestamp: DateTime<Local>,
}
