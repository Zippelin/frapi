use std::sync::{Arc, Mutex};

use futures::executor::block_on;
use serde_json::Value;

use crate::{
    executor::{Executor, State},
    settings::{
        main_settings::entity::request_settings::{
            request_setup_settings::RequestSetupSettings, RequestSettings,
        },
        HttpVersionSetting, RequestHttpSetupSettings, RequestWsSetupSettings,
    },
    states::{
        main_page::{
            generics::{CountedText, Header},
            request::request_data::{BodyFromData, FormFieldType, RequestData},
            response::Response,
        },
        Events,
    },
};

pub mod request_data;

/// Request Entity state represenation
#[derive(Debug, Clone)]
pub struct Request {
    /// unique ID of request UUIDv4
    pub id: String,
    // mark if entity was changed somehow, excluding responses
    pub is_changed: bool,
    /// original request data, before save
    pub original: RequestData,
    /// draft request data, currently alloweed for edit
    pub draft: RequestData,
    /// vec of responses
    pub responses: Arc<Mutex<Vec<Response>>>,
    /// new header data - for UI, to add new neader
    pub new_header: Header,
    /// new form data - for UI, to add new field
    pub new_body_form_field: BodyFromData,
    /// request executor engine
    pub executor: Executor,
    /// details or request currently on screen
    pub visible_details: RequestDetails,
    /// headers currently visible on screen
    pub visible_headers: RequestHeaders,
    pub visible_body: RequestBodyDetails,
}

/// From Settings -> State
impl From<&RequestSettings> for Request {
    fn from(value: &RequestSettings) -> Self {
        let responses: Arc<Mutex<Vec<Response>>> = Arc::new(Mutex::new(vec![]));
        let response_for_executer: Arc<Mutex<Vec<Response>>> = Arc::clone(&responses);

        let original = RequestData::from(value);
        let draft = RequestData::from(value);

        let visible_details = if draft.protocot_is_http() {
            RequestDetails::QueryParams
        } else if draft.protocot_is_ws() {
            RequestDetails::Message
        } else {
            RequestDetails::Header
        };
        Self {
            id: value.id.clone(),
            is_changed: false,
            original,
            draft,
            responses,
            executor: Executor::new(response_for_executer),
            new_header: Header {
                key: "".into(),
                value: "".into(),
            },
            visible_details,
            visible_headers: RequestHeaders::Custom,
            visible_body: RequestBodyDetails::Raw,
            new_body_form_field: BodyFromData {
                key: "".into(),
                value: "".into(),
                field_type: FormFieldType::Text,
            },
        }
    }
}

impl Request {
    pub fn default() -> Self {
        let responses = Arc::new(Mutex::new(vec![]));
        let executor = Executor::new(responses.clone());
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            is_changed: false,
            original: RequestData::default(),
            draft: RequestData::default(),
            responses,
            new_header: Header::default(),
            executor,
            visible_details: RequestDetails::Header,
            visible_headers: RequestHeaders::Custom,
            visible_body: RequestBodyDetails::Raw,
            new_body_form_field: BodyFromData {
                key: "".into(),
                value: "".into(),
                field_type: FormFieldType::Text,
            },
        }
    }
    /// Fire Executor to make requests
    pub fn go(&mut self, events: Arc<Mutex<Events>>, delay_send_message: bool) {
        // Clearing body parts wich does not selected currently
        let mut request_data = self.draft.clone();
        match self.visible_body {
            RequestBodyDetails::Raw => {
                request_data.body.binary_path = "".into();
                request_data.body.form_data = vec![];
            }
            RequestBodyDetails::FormData => {
                request_data.body.binary_path = "".into();
                request_data.body.raw = CountedText::default();
            }
            RequestBodyDetails::Binary => {
                request_data.body.form_data = vec![];
                request_data.body.raw = CountedText::default();
            }
        }
        self.executor
            .execute(&request_data, delay_send_message, events);
    }

    /// Stop Executor, also drop execution in progress
    /// This function block main thread to wait termination ends.
    /// Since termination very simple send to channel method, real block wont happen
    pub fn termiate(&mut self) {
        block_on(self.executor.terminate());
    }

    /// Check if Executor is Free for job
    pub fn executor_is_free(&self) -> bool {
        let lock_executor_state = self.executor.state.lock();
        let state = lock_executor_state.unwrap();

        if state.eq(&State::FREE) {
            return true;
        }
        return false;
    }

    /// Drop change mark and transfer Draft to Original Data.
    /// Will check if changes accured and save realy need to be.
    /// For Save purpose.
    pub fn on_save(&mut self) {
        if self.is_changed {
            self.is_changed = false;
            self.original.copy_from_other(&self.draft);
        }
    }

    /// Check if filter string could be applied here
    pub fn is_filtered(&self, filter: &String) -> bool {
        if self
            .draft
            .name
            .to_lowercase()
            .contains(&filter.to_lowercase())
            || self
                .draft
                .body
                .raw
                .message
                .to_lowercase()
                .contains(&filter.to_lowercase())
            || self
                .draft
                .protocol
                .to_string()
                .to_lowercase()
                .contains(&filter.to_lowercase())
            || self
                .draft
                .method
                .to_string()
                .to_lowercase()
                .contains(&filter.to_lowercase())
            || self
                .draft
                .uri
                .to_lowercase()
                .contains(&filter.to_lowercase())
        {
            return true;
        }
        false
    }

    pub fn cancel_changes(&mut self) {
        self.draft.copy_from_other(&self.original);
        self.is_changed = false
    }

    pub fn details_to_header(&mut self) {
        self.visible_details = RequestDetails::Header
    }

    pub fn details_to_body(&mut self) {
        self.visible_details = RequestDetails::Body
    }

    /// Parse Url string
    pub fn parse_url(&mut self) {
        self.draft.parse_url();
    }

    pub fn prettier_ws_message(&mut self) {
        let parser = serde_json::from_str::<Value>(&self.draft.message.message);

        if parser.is_err() {
            return;
        }

        let value = parser.unwrap();

        let prettier = serde_json::to_string_pretty(&value);

        if let Ok(pretty_string) = prettier {
            self.draft.message.set(pretty_string);
        }
    }
}

/// Request details currently shown on UI
#[derive(Debug, Clone, PartialEq)]
pub enum RequestDetails {
    Header,
    Body,
    QueryParams,
    Message,
    Setup,
}

/// Request details currently shown on UI
#[derive(Debug, Clone, PartialEq)]
pub enum RequestBodyDetails {
    Raw,
    FormData,
    Binary,
}

/// Request details currently shown on UI
#[derive(Debug, Clone, PartialEq)]
pub enum RequestHeaders {
    Default,
    Custom,
}

/// Default Headers for WS - cant be changed!
pub fn default_ws_headers() -> Vec<Header> {
    vec![
        Header {
            key: "Host".into(),
            value: "<calculated...>".into(),
        },
        Header {
            key: "Connection".into(),
            value: "Upgrade".into(),
        },
        Header {
            key: "Upgrade".into(),
            value: "websocket".into(),
        },
        Header {
            key: "Sec-WebSocket-Version".into(),
            value: "13".into(),
        },
        Header {
            key: "Sec-WebSocket-Key".into(),
            value: "<calculated...>".into(),
        },
    ]
}

#[derive(Debug, Clone, PartialEq)]
pub enum HttpVersion {
    AUTO,
    HTTPv1,
    HTTPv2,
}

impl HttpVersion {
    pub fn to_string(&self) -> String {
        match self {
            HttpVersion::AUTO => "AUTO".into(),
            HttpVersion::HTTPv1 => "HTTPv1".into(),
            HttpVersion::HTTPv2 => "HTTPv2".into(),
        }
    }
}

/// Settings to make reqeust
#[derive(Debug, Clone)]
pub enum RequestSetup {
    HTTP(RequestHttpSetup),
    WS(RequestWsSetup),
}

impl RequestSetup {
    pub fn http(&self) -> Option<&RequestHttpSetup> {
        match self {
            RequestSetup::HTTP(request_http_setup) => Some(request_http_setup),
            RequestSetup::WS(_) => None,
        }
    }

    pub fn ws(&self) -> Option<&RequestWsSetup> {
        match self {
            RequestSetup::HTTP(_) => None,
            RequestSetup::WS(request_ws_setup) => Some(request_ws_setup),
        }
    }

    pub fn http_mut(&mut self) -> Option<&mut RequestHttpSetup> {
        match self {
            RequestSetup::HTTP(request_http_setup) => Some(request_http_setup),
            RequestSetup::WS(_) => None,
        }
    }

    pub fn ws_mut(&mut self) -> Option<&mut RequestWsSetup> {
        match self {
            RequestSetup::HTTP(_) => None,
            RequestSetup::WS(request_ws_setup) => Some(request_ws_setup),
        }
    }

    pub fn default_ws() -> Self {
        Self::WS(RequestWsSetup::default())
    }
}

impl From<&RequestSetupSettings> for RequestSetup {
    fn from(value: &RequestSetupSettings) -> Self {
        match value {
            RequestSetupSettings::HTTP(request_http_setup_settings) => {
                Self::HTTP(RequestHttpSetup::from(request_http_setup_settings))
            }
            RequestSetupSettings::WS(request_ws_setup_settings) => {
                Self::WS(RequestWsSetup::from(request_ws_setup_settings))
            }
        }
    }
}

impl Default for RequestSetup {
    fn default() -> Self {
        Self::HTTP(RequestHttpSetup::default())
    }
}

/// Settings to make ws reqeust
#[derive(Debug, Clone)]
pub struct RequestWsSetup {
    pub reconnection_timeout: String,
    pub reconnection_attempts: String,
}

impl From<&RequestWsSetupSettings> for RequestWsSetup {
    fn from(value: &RequestWsSetupSettings) -> Self {
        Self {
            reconnection_timeout: value.reconnection_timeout.to_string(),
            reconnection_attempts: value.reconnection_attempts.to_string(),
        }
    }
}

impl Default for RequestWsSetup {
    fn default() -> Self {
        Self {
            reconnection_timeout: "5000".into(),
            reconnection_attempts: "3".into(),
        }
    }
}

/// Settings to make http reqeust
#[derive(Debug, Clone)]
pub struct RequestHttpSetup {
    pub http_version: HttpVersion,
    pub use_cookies: bool,
    pub use_redirects: bool,
    pub redirects_amount: String,
}

impl Default for RequestHttpSetup {
    fn default() -> Self {
        Self {
            http_version: HttpVersion::AUTO,
            use_cookies: true,
            use_redirects: true,
            redirects_amount: "9".into(),
        }
    }
}

impl From<&RequestHttpSetupSettings> for RequestHttpSetup {
    fn from(value: &RequestHttpSetupSettings) -> Self {
        let http_version = match value.http_version {
            HttpVersionSetting::HTTPv1 => HttpVersion::HTTPv1,
            HttpVersionSetting::HTTPv2 => HttpVersion::HTTPv2,
            HttpVersionSetting::AUTO => HttpVersion::AUTO,
        };
        Self {
            http_version,
            use_cookies: value.use_cookies,
            use_redirects: value.use_redirects,
            redirects_amount: value.redirects_amount.to_string(),
        }
    }
}
