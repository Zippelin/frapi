use serde::{Deserialize, Serialize};

use crate::states::{Style, Theme};

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, Default)]
pub struct UISettings {
    pub theme: UITheme,
}

impl From<&Style> for UISettings {
    fn from(value: &Style) -> Self {
        let theme = match &value.theme {
            Theme::Light(_) => UITheme::Light,
            Theme::Dark(_) => UITheme::Dark,
        };
        Self { theme }
    }
}

impl UISettings {
    pub fn default() -> Self {
        Self {
            theme: UITheme::Dark,
        }
    }
}

// Settings::Theme - color theme for application
#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, Default)]
pub enum UITheme {
    Light,
    #[default]
    Dark,
}
