use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    settings::main_settings::entity::request_settings::{
        body_settings::RequestBodySettigns, method_settigns::Method, protocol_settings::Protocol,
        request_setup_settings::RequestSetupSettings,
    },
    states::main_page::{generics::Header as StateHeader, request::Request as StateRequest},
};

pub mod body_settings;
pub mod method_settigns;
pub mod protocol_settings;
pub mod request_setup_settings;

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, Default)]
pub struct RequestSettings {
    pub id: String,
    pub name: String,
    pub protocol: Protocol,
    pub method: Method,
    pub uri: String,
    pub headers: Vec<Header>,
    pub body: RequestBodySettigns,
    pub message: String,
    pub setup: RequestSetupSettings,
}

impl From<&StateRequest> for RequestSettings {
    fn from(value: &StateRequest) -> Self {
        let headers = value
            .draft
            .headers
            .iter()
            .map(|val| Header::from(val))
            .collect();
        Self {
            id: value.id.clone(),
            name: value.draft.name.clone(),
            protocol: value.draft.protocol.clone(),
            method: value.draft.method.clone(),
            uri: value.draft.uri.clone(),
            headers,
            body: RequestBodySettigns::from(&value.draft.body),
            message: value.draft.message.message.clone(),
            setup: RequestSetupSettings::from(&value.draft.setup),
        }
    }
}

impl RequestSettings {
    pub fn from_original(value: &StateRequest) -> Self {
        let headers = value
            .original
            .headers
            .iter()
            .map(|val| Header::from(val))
            .collect();
        Self {
            id: value.id.clone(),
            name: value.original.name.clone(),
            protocol: value.original.protocol.clone(),
            method: value.original.method.clone(),
            uri: value.original.uri.clone(),
            headers,
            body: RequestBodySettigns::from(&value.original.body),
            message: value.original.message.message.clone(),
            setup: RequestSetupSettings::from(&value.original.setup),
        }
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, Default)]
pub struct Header {
    pub key: String,
    pub value: Value,
}

impl From<&StateHeader> for Header {
    fn from(value: &StateHeader) -> Self {
        match serde_json::from_str(&value.value) {
            Ok(val) => Self {
                key: value.key.clone(),
                value: val,
            },
            Err(_) => Self {
                key: value.key.clone(),
                value: Value::from(value.value.clone()),
            },
        }
    }
}
