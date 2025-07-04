use std::sync::{Arc, Mutex};

use chrono::{DateTime, Local};

use serde_json::Value;
use tokio_tungstenite::tungstenite::Utf8Bytes;

use crate::{
    executor::Executor,
    settings::{
        CollectionSettings, Entity as SettingsEntity, Method, Protocol, RequestSettings, Settings,
    },
    states::Style,
};

use reqwest::{header::HeaderMap, Error, Response as HttpResponse};

#[derive(Debug)]
pub enum Entity {
    COLLECTION(Collection),
    REQUEST(Request),
}

impl From<&SettingsEntity> for Entity {
    fn from(value: &SettingsEntity) -> Self {
        match value {
            SettingsEntity::COLLECTION(collection_settings) => {
                Self::COLLECTION(Collection::from(collection_settings))
            }
            SettingsEntity::REQUEST(request_settings) => {
                Self::REQUEST(Request::from(request_settings))
            }
        }
    }
}

#[derive(Debug)]
pub struct Request {
    pub id: String,
    pub is_changed: bool,
    pub original: RequestData,
    pub draft: RequestData,
    pub responses: Arc<Mutex<Vec<Response>>>,
    executor: Executor,
}

impl From<&RequestSettings> for Request {
    fn from(value: &RequestSettings) -> Self {
        let responses: Arc<Mutex<Vec<Response>>> = Arc::new(Mutex::new(vec![]));
        let response_for_executer: Arc<Mutex<Vec<Response>>> = Arc::clone(&responses);

        let original = RequestData::from(value);
        let draft = RequestData::from(value);
        Self {
            id: value.id.clone(),
            is_changed: false,
            original,
            draft,
            responses,
            executor: Executor::new(response_for_executer),
        }
    }
}

impl Request {
    pub fn go(&mut self) {
        self.executor.execute(&self.draft);
    }

    pub fn termiate(&mut self) {
        self.executor.terminate();
    }
}

#[derive(Debug)]
pub struct Collection {
    pub id: String,
    pub is_changed: bool,
    pub original: CollectionData,
    pub draft: CollectionData,
    pub requests: Vec<Request>,
}

impl From<&CollectionSettings> for Collection {
    fn from(value: &CollectionSettings) -> Self {
        let original: CollectionData = CollectionData::from(value);
        let draft = CollectionData::from(value);

        let mut requests = vec![];
        for request in &value.requests {
            requests.push(Request::from(request));
        }
        Self {
            id: value.id.clone(),
            is_changed: false,
            original,
            draft,
            requests,
        }
    }
}

#[derive(Debug)]
pub struct Header {
    pub key: String,
    pub value: Value,
}

#[derive(Debug)]
pub struct RequestData {
    pub name: String,
    pub protocol: Protocol,
    pub method: Method,
    pub uri: String,
    pub headers: Vec<Header>,
    pub body: String,
}

impl From<&RequestSettings> for RequestData {
    fn from(value: &RequestSettings) -> Self {
        let mut headers = vec![];
        for header in &value.headers {
            headers.push(Header {
                key: header.key.clone(),
                value: header.value.clone(),
            });
        }
        Self {
            name: value.name.clone(),
            protocol: value.protocol.clone(),
            method: value.method.clone(),
            uri: value.uri.clone(),
            headers,
            body: "".into(),
        }
    }
}

#[derive(Debug)]
pub struct CollectionData {
    pub name: String,
    pub description: String,
}

impl From<&CollectionSettings> for CollectionData {
    fn from(value: &CollectionSettings) -> Self {
        Self {
            name: value.name.clone(),
            description: value.description.clone(),
        }
    }
}

#[derive(Debug)]
pub struct SelectedEntity {
    pub collection_idx: Option<usize>,
    pub request_idx: Option<usize>,
}

impl SelectedEntity {
    pub fn new() -> Self {
        Self {
            collection_idx: None,
            request_idx: None,
        }
    }

    pub fn select_request(mut self, idx: usize) -> Self {
        self.request_idx = Some(idx);
        self
    }

    pub fn select_collection(mut self, idx: usize) -> Self {
        self.collection_idx = Some(idx);
        self
    }

    pub fn uselect_request(mut self) -> Self {
        self.request_idx = None;
        self
    }

    pub fn uselect_collection(mut self) -> Self {
        self.collection_idx = None;
        self
    }

    pub fn is_selected(&self) -> bool {
        if self.collection_idx.is_none() && self.request_idx.is_none() {
            return false;
        }
        true
    }
}

#[derive(Debug)]
pub struct Response {
    pub time: DateTime<Local>,
    pub data: ResponseData,
    pub selected_view: ResponseView,
    pub code: usize,
}

impl Response {
    pub fn from_utf8_bytes(data: Utf8Bytes) -> Self {
        let json: Value = match serde_json::from_str(&data.to_string()) {
            Ok(val) => val,
            Err(_) => Value::Null,
        };

        Self {
            time: Local::now(),
            data: ResponseData {
                raw: data.to_string(),
                json,
                headers: vec![],
            },
            selected_view: ResponseView::RAW,
            code: 0,
        }
    }
    pub async fn from_http_response(http_response: HttpResponse) -> Self {
        match http_response.error_for_status() {
            Ok(result) => {
                let mut headers = vec![];

                for header in result.headers() {
                    headers.push(Header {
                        key: header.0.to_string(),
                        value: header.1.as_bytes().into(),
                    });
                }

                let code = result.status().as_u16() as usize;

                let text: String = result
                    .text()
                    .await
                    .unwrap_or("Error during text receiving".into());

                let json: Value = match serde_json::from_str(&text) {
                    Ok(val) => val,
                    Err(_) => Value::Null,
                };

                let data = ResponseData {
                    raw: text,
                    json,
                    headers,
                };
                Self {
                    time: Local::now(),
                    data: data,
                    selected_view: ResponseView::RAW,
                    code,
                }
            }

            Err(err) => {
                let raw = err
                    .status()
                    .expect("Error during text receiving")
                    .canonical_reason()
                    .expect("Error during text receiving")
                    .to_string();

                let data = ResponseData {
                    raw,
                    json: Value::Null,
                    headers: vec![],
                };

                Self {
                    time: Local::now(),
                    data: data,
                    selected_view: ResponseView::RAW,
                    code: err.status().unwrap().as_u16() as usize,
                }
            }
        }
    }

    pub async fn from_http_error(error: Error) -> Self {
        let raw = match error.status() {
            Some(val) => val
                .canonical_reason()
                .expect("During Reqeust Error occured. Could not read error reason.")
                .to_string(),
            None => "During Reqeust Error occured. Could not read error reason.".to_string(),
        };

        let code = match error.status() {
            Some(val) => val.as_u16() as usize,
            None => 0,
        };

        let data = ResponseData {
            raw: raw,
            json: Value::Null,
            headers: vec![],
        };
        Self {
            time: Local::now(),
            data,
            selected_view: ResponseView::RAW,
            code,
        }
    }
}

#[derive(Debug)]
pub struct ResponseData {
    pub raw: String,
    pub json: Value,
    pub headers: Vec<Header>,
}

#[derive(Debug)]
pub enum ResponseView {
    JSON,
    RAW,
    HEADERS,
}

#[derive(Debug)]
pub struct MainPage {
    pub entities: Vec<Entity>,
    pub filtered_entities: Vec<usize>,
    pub selected_entity: SelectedEntity,
    pub style: Style,
}

impl From<&Settings> for MainPage {
    fn from(value: &Settings) -> Self {
        let mut entities = vec![];
        for entity in &value.main_page.entities {
            entities.push(Entity::from(entity));
        }
        Self {
            filtered_entities: (0..entities.len()).collect(),
            entities,
            selected_entity: SelectedEntity::new(),
            style: Style::from(&value.ui),
        }
    }
}

impl MainPage {
    pub fn selected_as_changed(&mut self) {
        if !self.selected_entity.is_selected() {
            return;
        };

        if self.selected_entity.collection_idx.is_none() {
            let entity = self
                .entities
                .get_mut(self.selected_entity.request_idx.unwrap())
                .unwrap();

            match entity {
                Entity::COLLECTION(_collection) => {}
                Entity::REQUEST(request) => request.is_changed = true,
            }
            return;
        };

        if self.selected_entity.collection_idx.is_some()
            && self.selected_entity.request_idx.is_none()
        {
            let entity = self
                .entities
                .get_mut(self.selected_entity.collection_idx.unwrap())
                .unwrap();

            match entity {
                Entity::COLLECTION(collection) => collection.is_changed = true,
                Entity::REQUEST(_request) => {}
            }
            return;
        };

        if self.selected_entity.collection_idx.is_some()
            && self.selected_entity.request_idx.is_some()
        {
            let entity = self
                .entities
                .get_mut(self.selected_entity.collection_idx.unwrap())
                .unwrap();

            match entity {
                Entity::COLLECTION(collection) => {
                    collection
                        .requests
                        .get_mut(self.selected_entity.request_idx.unwrap())
                        .unwrap()
                        .is_changed = true
                }
                Entity::REQUEST(_request) => {}
            }
        };
    }
}
