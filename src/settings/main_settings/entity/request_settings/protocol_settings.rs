use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, Default)]
pub enum Protocol {
    HTTP,
    #[default]
    HTTPS,
    WS,
    WSS,
}

impl Display for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Protocol::HTTP => write!(f, "HTTP"),
            Protocol::HTTPS => write!(f, "HTTPS"),
            Protocol::WS => write!(f, "WS"),
            Protocol::WSS => write!(f, "WSS"),
        }
    }
}
