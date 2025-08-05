use serde::{Deserialize, Serialize};

use crate::{
    settings::{RequestHttpSetupSettings, RequestWsSetupSettings},
    states::main_page::request::RequestSetup,
};

pub mod http_setup_settings;
pub mod ws_setup_settigns;

/// Settings to make reqeust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RequestSetupSettings {
    HTTP(RequestHttpSetupSettings),
    WS(RequestWsSetupSettings),
}

impl Default for RequestSetupSettings {
    fn default() -> Self {
        Self::HTTP(RequestHttpSetupSettings::default())
    }
}

impl From<&RequestSetup> for RequestSetupSettings {
    fn from(value: &RequestSetup) -> Self {
        match value {
            RequestSetup::HTTP(request_http_setup) => {
                Self::HTTP(RequestHttpSetupSettings::from(request_http_setup))
            }
            RequestSetup::WS(request_ws_setup) => {
                Self::WS(RequestWsSetupSettings::from(request_ws_setup))
            }
        }
    }
}

impl RequestSetupSettings {
    pub fn http() -> Self {
        Self::HTTP(RequestHttpSetupSettings::default())
    }
    pub fn ws() -> Self {
        Self::WS(RequestWsSetupSettings::default())
    }
}
