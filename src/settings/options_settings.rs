use serde::{Deserialize, Serialize};

use crate::states::Options as StateOptions;

// Settings::Options - general settings
#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, Default)]
pub struct OptionsSettings {
    pub window_size: (f32, f32),
    pub window_position: Option<(f32, f32)>,
}

impl From<&StateOptions> for OptionsSettings {
    fn from(value: &StateOptions) -> Self {
        let window_position = if let Some(pos) = value.window_position {
            Some((pos.x, pos.y))
        } else {
            None
        };

        Self {
            window_size: (value.window_size.x, value.window_size.y),
            window_position,
        }
    }
}

impl OptionsSettings {
    pub fn default() -> Self {
        Self {
            window_size: (800., 600.),
            window_position: None,
        }
    }
}
