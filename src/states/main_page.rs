/// Stated of Main page of application
///
use std::{
    hash::{DefaultHasher, Hash, Hasher},
    sync::{Arc, Mutex},
};

use chrono::{DateTime, Local};

use serde_json::Value;
use tokio_tungstenite::tungstenite::Utf8Bytes;

use crate::{
    executor::{Executor, State},
    settings::{
        CollectionSettings, Entity as SettingsEntity, Method, Protocol, RequestSettings, Settings,
    },
    states::{Events, Style},
};

use reqwest::{Error, Response as HttpResponse};

/// Typoes of Entities allowed on left side of Panel.
/// This could be Collection, containing other Reqeusts or Request on root level
#[derive(Debug, Clone)]
pub enum Entity {
    COLLECTION(Collection),
    REQUEST(Request),
}

/// From Settings -> State
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
    executor: Executor,
}

/// From Settings -> State
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
            new_header: Header {
                key: "".into(),
                value: "".into(),
            },
        }
    }
}

impl Request {
    /// Fire Executor to make reqeusts
    pub fn go(&mut self, events: Arc<Mutex<Events>>) {
        self.executor.execute(&self.draft, events);
    }

    /// Stop Executor, also drop execution in progress
    pub fn termiate(&mut self) {
        self.executor.terminate();
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
}

/// Collection Entity representation
#[derive(Debug, Clone)]
pub struct Collection {
    // unique ID of request UUIDv4
    pub id: String,
    // mark if entity was changed somehow, excluding reqeust delete/add
    pub is_changed: bool,
    /// original collection data, before save
    pub original: CollectionData,
    /// draft collection data, currently alloweed for edit
    pub draft: CollectionData,
    /// vec of requests
    pub requests: Vec<Request>,
    /// state if collection visualy folded or not
    pub is_folded: bool,
}

/// From Settigns -> State
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
            is_folded: true,
        }
    }
}

impl Collection {
    /// Drop change mark and transfer Draft to Original Data.
    /// Will check if changes accured and save realy need to be.
    /// For Save purpose.
    pub fn on_save(&mut self) {
        if self.is_changed {
            self.is_changed = false;
            self.original.copy_from_other(&self.draft);
        }
    }
}

/// Header of Request State
#[derive(Debug, Clone)]
pub struct Header {
    pub key: String,
    pub value: String,
}

/// Request data representation
#[derive(Debug, Clone)]
pub struct RequestData {
    pub name: String,
    pub protocol: Protocol,
    pub method: Method,
    pub uri: String,
    pub headers: Vec<Header>,
    pub body: String,
}

/// From Settigns -> State
impl From<&RequestSettings> for RequestData {
    fn from(value: &RequestSettings) -> Self {
        let mut headers = vec![];

        for header in &value.headers {
            // Remove redundant quotes from only string-like values
            match header.value.as_str() {
                Some(val) => {
                    headers.push(Header {
                        key: header.key.clone(),
                        value: val.to_string(),
                    });
                }
                None => {
                    headers.push(Header {
                        key: header.key.clone(),
                        value: header.value.to_string(),
                    });
                }
            };
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

impl RequestData {
    /// Copy from other Self.
    /// Usualy used on Save.
    pub fn copy_from_other(&mut self, other_request: &Self) {
        self.name = other_request.name.clone();
        self.protocol = other_request.protocol.clone();
        self.method = other_request.method.clone();
        self.uri = other_request.uri.clone();
        self.body = other_request.body.clone();

        self.headers = other_request
            .headers
            .iter()
            .map(|val| Header {
                key: val.key.clone(),
                value: val.value.clone(),
            })
            .collect();
    }
}

/// Collection data representation
#[derive(Debug, Clone)]
pub struct CollectionData {
    pub name: String,
    pub description: String,
}

/// From Settings -> State
impl From<&CollectionSettings> for CollectionData {
    fn from(value: &CollectionSettings) -> Self {
        Self {
            name: value.name.clone(),
            description: value.description.clone(),
        }
    }
}

impl CollectionData {
    /// Copy from other Self.
    /// Usualy used on Save.
    pub fn copy_from_other(&mut self, other_collection: &Self) {
        self.name = other_collection.name.clone();
        self.description = other_collection.description.clone();
    }
}

/// Representations of currently selected value on left panel.
/// Contain Options of indexes of collection and request.
/// If Request on root level, collection will be None, otherwise collection will have Some index of parent fro selected request.
/// If Selectted only collection, but not request - reqeust index will be None
#[derive(Debug, Clone)]
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

    /// Selecting passes request
    pub fn select_request(&mut self, parent_collection_idx: Option<usize>, request_idx: usize) {
        self.request_idx = Some(request_idx);
        self.collection_idx = parent_collection_idx;
    }

    /// Selecting passes collection
    /// Automaticly uselect request
    pub fn select_collection(&mut self, idx: usize) {
        self.collection_idx = Some(idx);
        self.unselect_request();
    }

    /// Uselect request
    fn unselect_request(&mut self) {
        self.request_idx = None;
    }

    /// Uselect collection
    fn unselect_collection(&mut self) {
        self.collection_idx = None;
    }

    /// Check if collection or request is selected
    /// Mostly this state could be only after load, befote user start interract with UI
    pub fn is_selected(&self) -> bool {
        if self.collection_idx.is_none() && self.request_idx.is_none() {
            return false;
        }
        true
    }

    /// Check if collection is selected
    pub fn collection_is_selected(&self, idx: usize) -> bool {
        if let Some(collection_idx) = self.collection_idx {
            self.request_idx.is_none() && collection_idx == idx
        } else {
            false
        }
    }

    /// Check if request is selected
    pub fn request_is_selected(
        &self,
        parent_collection_idx: Option<usize>,
        request_idx: usize,
    ) -> bool {
        match parent_collection_idx {
            Some(val) => {
                if self.collection_idx.is_none() || self.request_idx.is_none() {
                    return false;
                };

                let current_collection_idx = self.collection_idx.unwrap();
                let current_request_idx = self.request_idx.unwrap();

                if current_collection_idx != val {
                    return false;
                };

                if current_request_idx != request_idx {
                    return false;
                }
                true
            }
            None => {
                if let Some(r_idx) = self.request_idx {
                    r_idx == request_idx
                } else {
                    false
                }
            }
        }
    }
}

/// Reqeust response state representation
#[derive(Debug, Clone)]
pub struct Response {
    /// local time response received
    pub time: DateTime<Local>,
    /// response data details
    pub data: ResponseData,
    /// representation of selected view for UI
    pub selected_view: ResponseView,
    /// HTTP code
    pub code: usize,
    /// state of UI element of folded
    pub is_folded: bool,
    // TODO: add how long request took. Troubles how to save request send time.
}

impl Response {
    /// Used as answear from WSS ot WS
    pub fn from_utf8_bytes(data: Utf8Bytes) -> Self {
        Self {
            time: Local::now(),
            data: ResponseData::new(data.to_string(), vec![]),
            selected_view: ResponseView::RAW,
            code: 0,
            is_folded: true,
        }
    }

    /// Used as answear from HTTP or HTTPS
    pub async fn from_http_response(http_response: HttpResponse) -> Result<Self, (Self, String)> {
        match http_response.error_for_status() {
            Ok(result) => {
                let mut headers = vec![];

                for header in result.headers() {
                    headers.push(Header {
                        key: header.0.to_string(),
                        value: header.1.to_str().unwrap().to_string(),
                    });
                }

                let code = result.status().as_u16() as usize;

                match result.text().await {
                    Ok(text) => Ok(Self {
                        time: Local::now(),
                        data: ResponseData::new(text, vec![]),
                        selected_view: ResponseView::RAW,
                        code,
                        is_folded: true,
                    }),
                    Err(err) => Err((
                        Self {
                            time: Local::now(),
                            data: ResponseData::new("Error during text receiving".into(), vec![]),
                            selected_view: ResponseView::RAW,
                            code,
                            is_folded: true,
                        },
                        format!("Error during text receiving. Error: {}", err),
                    )),
                }
            }

            Err(err) => match err.status() {
                Some(err_status_code) => match err_status_code.canonical_reason() {
                    Some(err_reason) => Ok(Self {
                        time: Local::now(),
                        data: ResponseData::new(err_reason.into(), vec![]),
                        selected_view: ResponseView::RAW,
                        code: err.status().unwrap().as_u16() as usize,
                        is_folded: true,
                    }),
                    None => Err((
                        Self {
                            time: Local::now(),
                            data: ResponseData::new(
                                "Error during text receiving for error reason".into(),
                                vec![],
                            ),
                            selected_view: ResponseView::RAW,
                            code: err.status().unwrap().as_u16() as usize,
                            is_folded: true,
                        },
                        "Error during text receiving for error reason".into(),
                    )),
                },
                None => Err((
                    Self {
                        time: Local::now(),
                        data: ResponseData::new(
                            "Error during status code receiving".into(),
                            vec![],
                        ),
                        selected_view: ResponseView::RAW,
                        code: err.status().unwrap().as_u16() as usize,
                        is_folded: true,
                    },
                    "Error during text receiving".into(),
                )),
            },
        }
    }

    /// Used as answear from HTTP or HTTPS for HTTP errors
    pub async fn from_http_error(error: Error) -> Result<Self, (Self, String)> {
        let code = match error.status() {
            Some(val) => val.as_u16() as usize,
            None => 0,
        };

        match error.status() {
            Some(status_code) => match status_code.canonical_reason() {
                Some(canonical_reason) => Ok(Self {
                    time: Local::now(),
                    data: ResponseData::new(canonical_reason.into(), vec![]),
                    selected_view: ResponseView::RAW,
                    code,
                    is_folded: true,
                }),
                None => Err((
                    Self {
                        time: Local::now(),
                        data: ResponseData::new(
                            "During Request Error occured. Could not read error status code."
                                .into(),
                            vec![],
                        ),
                        selected_view: ResponseView::RAW,
                        code,
                        is_folded: true,
                    },
                    "During Request Error occured. Could not read error status code.".into(),
                )),
            },
            None => Err((
                Self {
                    time: Local::now(),
                    data: ResponseData::new(
                        "During Request Error occured. Could not read error reason.".into(),
                        vec![],
                    ),
                    selected_view: ResponseView::RAW,
                    code,
                    is_folded: true,
                },
                "During Request Error occured. Could not read error reason.".into(),
            )),
        }
    }
}

/// Json representations for UI
#[derive(Debug, Clone, PartialEq)]
pub enum JsonViewType {
    Simple,
    Comlex,
}

/// Json data
#[derive(Debug, Clone)]
pub struct JsonView {
    pub simple: String,
    pub complex: Value,
    pub view_type: JsonViewType,
}

impl JsonView {
    pub fn new(text: &String) -> Self {
        let (simple, complex) = match serde_json::from_str(text) {
            Ok(val) => {
                let beauty = serde_json::to_string_pretty(&val);
                if beauty.is_err() {
                    (text.clone(), val)
                } else {
                    (beauty.unwrap(), val)
                }
            }
            Err(_) => ("".into(), Value::Null),
        };
        Self {
            simple,
            complex,
            view_type: JsonViewType::Simple,
        }
    }
}

/// Response data state
#[derive(Debug, Clone)]
pub struct ResponseData {
    pub raw: String,
    pub json: JsonView,
    pub headers: Vec<Header>,
}

impl ResponseData {
    pub fn new(raw: String, headers: Vec<Header>) -> Self {
        Self {
            json: JsonView::new(&raw),
            raw,
            headers,
        }
    }

    /// Checking if JSON state exist for raw response data
    pub fn json_is_exist(&self) -> bool {
        !self.json.complex.is_null()
    }
}

/// Response representations types
#[derive(Debug, Clone, PartialEq)]
pub enum ResponseView {
    JSON,
    RAW,
    HEADERS,
}

#[derive(Debug, Clone)]
pub enum FilteredEntity {
    COLLECTION {
        collection_idx: usize,
        request_idxs: Vec<usize>,
    },
    REQUEST {
        reqeust_idx: usize,
    },
}

#[derive(Debug, Clone)]
pub struct FilteredEntities {
    pub items: Vec<FilteredEntity>,
}

impl From<&Vec<Entity>> for FilteredEntities {
    fn from(value: &Vec<Entity>) -> Self {
        let mut items = vec![];
        for i in 0..value.len() {
            match &value[i] {
                Entity::COLLECTION(collection) => items.push(FilteredEntity::COLLECTION {
                    collection_idx: i,
                    request_idxs: (0..collection.requests.len()).collect(),
                }),
                Entity::REQUEST(_) => items.push(FilteredEntity::REQUEST { reqeust_idx: i }),
            }
        }
        Self { items }
    }
}

/// Main Page States
#[derive(Debug, Clone)]
pub struct MainPage {
    pub entities: Vec<Entity>,
    pub filtered_entities: FilteredEntities,
    pub selected_entity: SelectedEntity,
    pub filter_text: String,
    pub style: Style,
}

/// From Settings -> State
impl From<&Settings> for MainPage {
    fn from(value: &Settings) -> Self {
        let mut entities = vec![];
        for entity in &value.main_page.entities {
            entities.push(Entity::from(entity));
        }
        Self {
            filtered_entities: FilteredEntities::from(&entities),
            entities,
            selected_entity: SelectedEntity::new(),
            style: Style::from(&value.ui),
            filter_text: "".into(),
        }
    }
}

impl MainPage {
    /// Check if currently selected request
    pub fn is_request_selected(&self) -> bool {
        match self.selected_entity.request_idx {
            Some(_) => true,
            None => false,
        }
    }

    /// Check if currently collection only selected
    pub fn is_collection_selected(&self) -> bool {
        match self.selected_entity.collection_idx {
            Some(_) => {
                if self.selected_entity.request_idx.is_none() {
                    return true;
                }
                return false;
            }
            None => false,
        }
    }

    /// Get currently selected collection as mutable
    pub fn selected_collection_mut(&mut self) -> Option<&mut Collection> {
        if self.selected_entity.collection_idx.is_none() {
            return None;
        };
        match &mut self.entities[self.selected_entity.collection_idx.unwrap()] {
            Entity::COLLECTION(collection) => Some(collection),
            Entity::REQUEST(_) => None,
        }
    }

    /// Get currently selected request as mutable
    pub fn selected_request_mut(&mut self) -> Option<&mut Request> {
        if self.selected_entity.request_idx.is_none() {
            return None;
        };

        let request_idx = self.selected_entity.request_idx.unwrap();
        match self.selected_entity.collection_idx {
            Some(val) => {
                if let Entity::COLLECTION(collection) = &mut self.entities[val] {
                    return Some(&mut collection.requests[request_idx]);
                } else {
                    return None;
                }
            }
            None => {
                if let Entity::REQUEST(request) = &mut self.entities[request_idx] {
                    return Some(request);
                } else {
                    return None;
                }
            }
        }
    }

    /// Get currently selected request as consistent id salt for UI salt
    pub fn selected_request_salt(&self) -> String {
        let collection_string = if self.selected_entity.collection_idx.is_none() {
            "None".to_string()
        } else {
            self.selected_entity.collection_idx.unwrap().to_string()
        };

        let request_string = if self.selected_entity.request_idx.is_none() {
            "None".to_string()
        } else {
            self.selected_entity.request_idx.unwrap().to_string()
        };
        format!("{collection_string}-{request_string}")
    }

    /// Change currently selected request
    pub fn request_mut(
        &mut self,
        collection_idx: Option<usize>,
        request_idx: usize,
    ) -> &mut Request {
        match collection_idx {
            Some(val) => {
                if let Entity::COLLECTION(collection) = &mut self.entities[val] {
                    return &mut collection.requests[request_idx];
                } else {
                    let col_idx_str = if collection_idx.is_none() {
                        "None".into()
                    } else {
                        format!("{}", collection_idx.unwrap())
                    };
                    panic!("Could not find Request item by index: {col_idx_str}-{request_idx}")
                }
            }
            None => {
                if let Entity::REQUEST(request) = &mut self.entities[request_idx] {
                    return request;
                } else {
                    let col_idx_str = if collection_idx.is_none() {
                        "None".into()
                    } else {
                        format!("{}", collection_idx.unwrap())
                    };
                    panic!("Could not find Request item by index: {col_idx_str}-{request_idx}")
                }
            }
        }
    }

    /// Set currently selected request or collection as changed
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

    pub fn filter_entities(&mut self) {
        // for i in 0..self.entities.len() {
        //     match &self.entities[i] {
        //         Entity::COLLECTION(collection) => {
        //             if collection.draft.name.contains(&self.filter_text)
        //                 || collection.draft.description.contains(&self.filter_text)
        //             {
        //                 self.filtered_entities.push(i);
        //             }
        //         }
        //         Entity::REQUEST(request) => todo!(),
        //     }
        // }
    }
}

/// Получаем хеш из строки.
/// Используется для генерации уникального состояния свернутости ColapsingHeaders
pub fn get_map_hash_from_str(text: String) -> u64 {
    let mut hasher = DefaultHasher::new();
    text.hash(&mut hasher);
    hasher.finish()
}
