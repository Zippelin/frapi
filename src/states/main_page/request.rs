use std::sync::{Arc, Mutex};

use futures::executor::block_on;
use serde_json::Value;

use crate::{
    executor::{Executor, State},
    settings::RequestSettings,
    states::{
        main_page::{generics::Header, request::request_data::RequestData, response::Response},
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
    /// hew header data - for UI, to add new neader
    pub new_header: Header,
    /// request executor engine
    pub executor: Executor,
    /// details or request currently on screen
    pub visible_details: RequestDetails,
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
        }
    }
    /// Fire Executor to make requests
    pub fn go(&mut self, events: Arc<Mutex<Events>>, delay_send_message: bool) {
        self.executor
            .execute(&self.draft, delay_send_message, events);
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
}
