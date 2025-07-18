/// Stated of Main page of application
///
use std::sync::{Arc, Mutex};

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

impl Entity {
    pub fn is_collection(&self) -> bool {
        match &self {
            Entity::COLLECTION(_) => true,
            Entity::REQUEST(_) => false,
        }
    }

    pub fn is_request(&self) -> bool {
        match &self {
            Entity::COLLECTION(_) => false,
            Entity::REQUEST(_) => true,
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
        }
    }
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
    pub fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            is_changed: false,
            original: CollectionData::default(),
            draft: CollectionData::default(),
            requests: vec![],
            is_folded: true,
        }
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
                .description
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
}

/// Header of Request State
#[derive(Debug, Clone)]
pub struct Header {
    pub key: String,
    pub value: String,
}

impl Header {
    pub fn default() -> Self {
        Self {
            key: "".into(),
            value: "".into(),
        }
    }
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
    pub fn default() -> Self {
        Self {
            name: "New Request".into(),
            protocol: Protocol::HTTP,
            method: Method::GET,
            uri: "".into(),
            headers: vec![],
            body: "".into(),
        }
    }
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
    pub fn default() -> Self {
        Self {
            name: "New Collection".into(),
            description: "".into(),
        }
    }
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
    #[allow(dead_code)]
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
                if self.collection_idx.is_some() {
                    return false;
                }
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

/// Representation of indexes of filtered entity - collection or request
#[derive(Debug, Clone)]
pub enum FilteredEntity {
    COLLECTION {
        collection_idx: usize,
        request_idxs: Vec<usize>,
    },
    REQUEST {
        request_idx: usize,
    },
}

impl FilteredEntity {
    /// Creating filtered collection entity
    pub fn collection(index: usize) -> Self {
        Self::COLLECTION {
            collection_idx: index,
            request_idxs: vec![],
        }
    }

    // Pushing request id to Self if Self is Collection
    pub fn push_request(&mut self, index: usize) {
        match self {
            FilteredEntity::COLLECTION {
                collection_idx: _,
                request_idxs,
            } => request_idxs.push(index),
            FilteredEntity::REQUEST { request_idx: _ } => return,
        }
    }

    // Creating filtered request entity
    pub fn request(index: usize) -> Self {
        Self::REQUEST { request_idx: index }
    }
}

/// List of filtered entities
#[derive(Debug, Clone)]
pub struct FilteredEntities {
    pub items: Vec<FilteredEntity>,
}

/// From Vec of Entities to Filtered Entities indexes
impl From<&Vec<Entity>> for FilteredEntities {
    fn from(value: &Vec<Entity>) -> Self {
        let mut items = vec![];
        for i in 0..value.len() {
            match &value[i] {
                Entity::COLLECTION(collection) => items.push(FilteredEntity::COLLECTION {
                    collection_idx: i,
                    request_idxs: (0..collection.requests.len()).collect(),
                }),
                Entity::REQUEST(_) => items.push(FilteredEntity::REQUEST { request_idx: i }),
            }
        }
        Self { items }
    }
}

impl FilteredEntities {
    pub fn new() -> Self {
        Self { items: vec![] }
    }

    /// Push to current list collection-like entity with index
    pub fn push_collection(&mut self, index: usize) {
        self.items.push(FilteredEntity::collection(index));
    }

    /// Push to current list request-like entity with index
    /// Collection index not mandatory. If request on root levelv - collection idx is None
    pub fn push_request(&mut self, request_index: usize, collection_idx: Option<usize>) {
        match collection_idx {
            Some(c_idx) => match self.items.get_mut(c_idx) {
                Some(filtered_entity) => match filtered_entity {
                    FilteredEntity::COLLECTION {
                        collection_idx: _,
                        request_idxs,
                    } => request_idxs.push(request_index),
                    FilteredEntity::REQUEST { request_idx: _ } => return,
                },
                None => {
                    let mut collection = FilteredEntity::collection(c_idx);
                    collection.push_request(request_index);
                    self.items.push(collection);
                }
            },
            None => {
                self.items.push(FilteredEntity::request(request_index));
            }
        }
    }

    /// Receiving all root level entities indexes
    pub fn root_entities_idxs(&self) -> Vec<usize> {
        self.items
            .iter()
            .map(|value| match value {
                FilteredEntity::COLLECTION {
                    collection_idx,
                    request_idxs: _,
                } => collection_idx.clone(),
                FilteredEntity::REQUEST { request_idx } => request_idx.clone(),
            })
            .collect()
    }

    /// Receiving all request indexes from collection with index
    pub fn collection_requests_idxs(&self, requested_collection_idx: usize) -> Option<Vec<usize>> {
        for entity in &self.items {
            match entity {
                FilteredEntity::COLLECTION {
                    collection_idx,
                    request_idxs,
                } => {
                    if *collection_idx == requested_collection_idx {
                        return Some(request_idxs.clone());
                    }
                }
                FilteredEntity::REQUEST { request_idx: _ } => {}
            }
        }
        None
    }
}

/// Request move target - used when need to transfer request between collections
#[derive(Debug, Clone)]
pub struct RequestMoveTarget {
    target: Vec<Option<usize>>,
}

impl RequestMoveTarget {
    pub fn new() -> Self {
        Self { target: vec![] }
    }

    pub fn move_planned(&self) -> bool {
        self.target.len() > 0
    }

    pub fn take(&mut self) -> Option<usize> {
        self.target.remove(0)
    }

    pub fn add(&mut self, target: Option<usize>) {
        self.target.push(target);
    }
}

/// Main Page States
#[derive(Debug, Clone)]
pub struct MainPage {
    /// List of all entities - collection adn reqeusts
    pub entities: Vec<Entity>,
    /// List of indexes for entities with contain filter string
    pub filtered_entities: FilteredEntities,
    /// Currently selected (clicked) entity
    pub selected_entity: SelectedEntity,
    /// Entity marked for deletion on next frame
    pub deletion_entity: SelectedEntity,
    /// Filter string
    pub filter_text: String,
    /// Description where to move entity
    pub request_move_target: RequestMoveTarget,
    /// Styles and colors for UI theme
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
            request_move_target: RequestMoveTarget::new(),
            deletion_entity: SelectedEntity::new(),
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

    /// Get currently selected collection
    pub fn selected_collection(&self) -> Option<&Collection> {
        if self.selected_entity.collection_idx.is_none() {
            return None;
        };
        match &self.entities[self.selected_entity.collection_idx.unwrap()] {
            Entity::COLLECTION(collection) => Some(&collection),
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

    /// Get currently selected request
    pub fn selected_request(&self) -> Option<&Request> {
        if self.selected_entity.request_idx.is_none() {
            return None;
        };

        let request_idx = self.selected_entity.request_idx.unwrap();
        match self.selected_entity.collection_idx {
            Some(val) => {
                if let Entity::COLLECTION(collection) = &self.entities[val] {
                    return Some(&collection.requests[request_idx]);
                } else {
                    return None;
                }
            }
            None => {
                if let Entity::REQUEST(request) = &self.entities[request_idx] {
                    return Some(&request);
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

    pub fn apply_filter(&mut self) {
        if self.filter_text == "" {
            self.filtered_entities = FilteredEntities::from(&self.entities);
            return;
        };
        let mut filtered_entities = FilteredEntities::new();
        for entity_idx in 0..self.entities.len() {
            match &self.entities[entity_idx] {
                Entity::COLLECTION(collection) => {
                    if collection.is_filtered(&self.filter_text) {
                        filtered_entities.push_collection(entity_idx);
                    }

                    for request_idx in 0..collection.requests.len() {
                        if collection.requests[request_idx].is_filtered(&self.filter_text) {
                            filtered_entities.push_request(request_idx, Some(entity_idx));
                        }
                    }
                }
                Entity::REQUEST(request) => {
                    if request.is_filtered(&self.filter_text) {
                        filtered_entities.push_request(entity_idx, None);
                    }
                }
            }
        }
        self.filtered_entities = filtered_entities;
    }

    /// Drop filter adn regenerate idexes to display
    pub fn drop_filter(&mut self) {
        self.filter_text = "".into();
        self.apply_filter();
    }

    /// Creating new collection relative to currently selected.
    /// New Collection will be next after selected collection, or in the end if request selected
    pub fn new_collection(&mut self) -> usize {
        let selected_collection_idx = self.selected_entity.collection_idx;
        match selected_collection_idx {
            Some(idx) => {
                if self.selected_entity.request_idx.is_some() {
                    self.entities
                        .push(Entity::COLLECTION(Collection::default()));
                    return self.entities.len() - 1;
                } else {
                    self.entities
                        .insert(idx + 1, Entity::COLLECTION(Collection::default()));
                    return idx + 1;
                }
            }
            None => {
                self.entities
                    .push(Entity::COLLECTION(Collection::default()));
                return self.entities.len() - 1;
            }
        };
    }

    /// Creating new request relative to currently selected.
    /// New request will be next after selected request, or in selected collection
    pub fn new_request(&mut self) -> (Option<usize>, usize) {
        let selected_collection_idx = self.selected_entity.collection_idx;
        let selected_request_idx = self.selected_entity.request_idx;

        if let Some(selected_collection_idx) = selected_collection_idx {
            // If selected entity is request in collection
            if let Some(selected_request_idx) = selected_request_idx {
                match &mut self.entities[selected_collection_idx] {
                    // Selected request in collection
                    Entity::COLLECTION(collection) => {
                        collection
                            .requests
                            .insert(selected_request_idx + 1, Request::default());
                        return (Some(selected_collection_idx), selected_request_idx + 1);
                    }
                    // This could be logical error, but still adding new request next to collection entity
                    Entity::REQUEST(_) => {
                        self.entities.insert(
                            selected_collection_idx + 1,
                            Entity::REQUEST(Request::default()),
                        );
                        return (None, selected_collection_idx + 1);
                    }
                }
            // If selected entity is collection (request not selected)
            // This mean that root element - collection selected
            } else {
                match &mut self.entities[selected_collection_idx] {
                    Entity::COLLECTION(collection) => {
                        collection.requests.push(Request::default());
                        return (Some(selected_collection_idx), collection.requests.len() - 1);
                    }
                    // This could be logical error, but still adding new request next to collection entity
                    Entity::REQUEST(_) => {
                        self.entities.insert(
                            selected_collection_idx + 1,
                            Entity::REQUEST(Request::default()),
                        );
                        return (None, selected_collection_idx + 1);
                    }
                }
            }
        };

        // If selected only request without collection - means request on root level
        if let Some(selected_request_idx) = selected_request_idx {
            self.entities.insert(
                selected_request_idx + 1,
                Entity::REQUEST(Request::default()),
            );
            return (None, selected_request_idx + 1);
        };

        // If nothing is selected, creating as last in root
        self.entities.push(Entity::REQUEST(Request::default()));
        return (None, self.entities.len() - 1);
    }

    /// Try to get collection by index and set fold state
    pub fn set_collection_fold_state(&mut self, index: usize, state: bool) {
        match &mut self.entities[index] {
            Entity::COLLECTION(collection) => collection.is_folded = state,
            Entity::REQUEST(_) => {}
        }
    }

    /// Get state of changes in selected entity
    pub fn entity_is_changed(&self) -> bool {
        if self.is_collection_selected() {
            return self.selected_collection().unwrap().is_changed;
        };
        if self.is_request_selected() {
            return self.selected_request().unwrap().is_changed;
        };
        false
    }

    /// Cancel changes of selected entity
    pub fn cancel_changes_of_selected_entity(&mut self) {
        if self.is_collection_selected() {
            self.selected_collection_mut().unwrap().cancel_changes();
        };
        if self.is_request_selected() {
            self.selected_request_mut().unwrap().cancel_changes();
        };
    }
    /// Try to move request to target collection.
    /// Checking if there is planned move.
    /// Return [true] if move was done
    pub fn update_request_move(&mut self) -> bool {
        if self.request_move_target.move_planned() {
            let requested_move = self.request_move_target.take();
            self.move_selected_request_to_collection(requested_move);
            return true;
        };
        false
    }

    /// Move reqeust to target collection.
    /// None collection index = root level
    pub fn move_selected_request_to_collection(&mut self, target_collection_idx: Option<usize>) {
        if !self.is_request_selected() {
            return;
        }

        // Check if was in root and reqeust in root
        if target_collection_idx.is_none() && self.selected_entity.collection_idx.is_none() {
            return;
        }

        // Safe check done above
        let request_idx = self.selected_entity.request_idx.unwrap();

        // Check requested and old collection - same
        if let Some(old_collection_idx) = self.selected_entity.collection_idx {
            if let Some(new_collection_idx) = target_collection_idx {
                if new_collection_idx == old_collection_idx {
                    return;
                }
            }
        }
        // Before changing list of entities we must clear current selectio
        // self.selected_entity.unselect_request();

        let request = if let Some(collection) = self.selected_collection_mut() {
            // if current selected request in collection
            collection.requests.remove(request_idx)
        } else {
            // if request in root level
            let selected_request = self.entities.remove(request_idx);
            if let Entity::REQUEST(sr) = selected_request {
                sr
            // Cant happen, since we checked before that we not in collection, but on root level.
            } else {
                return;
            }
        };

        if let Some(mut target_col_idx) = target_collection_idx {
            // When removed request on prev step vec indexes changed and shift happened
            // If old position was on root level and target AFTER it, we need consider vec changes of indexes
            if self.selected_entity.collection_idx.is_none()
                && target_col_idx > self.selected_entity.request_idx.unwrap()
            {
                target_col_idx -= 1;
            }

            match &mut self.entities[target_col_idx] {
                Entity::COLLECTION(collection) => {
                    // if move target - other collection
                    collection.requests.push(request);
                    self.selected_entity
                        .select_request(Some(target_col_idx), collection.requests.len() - 1);
                    collection.is_folded = false;
                }
                // Cant happen in usual way. But already removed reqeust before,
                // so maybe here could be troubles if UI lacking restrictions
                Entity::REQUEST(_) => {}
            }
        } else {
            self.entities.push(Entity::REQUEST(request));
            self.selected_entity
                .select_request(None, self.entities.len() - 1)
        }

        self.drop_filter();
    }

    /// Checking if root is collection
    pub fn root_entity_is_collection(&self, idx: usize) -> bool {
        if idx >= self.entities.len() {
            return false;
        };
        self.entities[idx].is_collection()
    }

    /// Checking if root is request
    pub fn root_entity_is_request(&self, idx: usize) -> bool {
        if idx >= self.entities.len() {
            return false;
        };
        self.entities[idx].is_request()
    }

    pub fn get_collection_mut(&mut self, idx: usize) -> Option<&mut Collection> {
        let entity = match self.entities.get_mut(idx) {
            Some(val) => val,
            None => return None,
        };

        match entity {
            Entity::COLLECTION(collection) => Some(collection),
            Entity::REQUEST(_) => None,
        }
    }

    pub fn get_reqeust_mut(&mut self, idx: usize) -> Option<&mut Request> {
        let entity = match self.entities.get_mut(idx) {
            Some(val) => val,
            None => return None,
        };

        match entity {
            Entity::COLLECTION(_) => None,
            Entity::REQUEST(request) => Some(request),
        }
    }

    /// Delete requested entity
    /// Return mark if changes made or not
    pub fn delete_marked_entity(&mut self) -> bool {
        if !self.deletion_entity.is_selected() {
            return false;
        }

        println!("start deletion");
        match self.deletion_entity.collection_idx {
            // If collection exist
            Some(collection_idx) => match self.deletion_entity.request_idx {
                // If requeust in collection
                Some(request_idx) => match &mut self.entities[collection_idx] {
                    // if buy collection_idx exist collection - success
                    Entity::COLLECTION(collection) => {
                        // if we select currently deleting entity - need to uselect
                        if self
                            .selected_entity
                            .request_is_selected(Some(collection_idx), request_idx)
                        {
                            self.selected_entity.unselect_request();
                        }
                        collection.requests.remove(request_idx);
                        self.drop_filter();
                        self.deletion_entity.unselect_request();
                        return true;
                    }
                    Entity::REQUEST(_) => return false,
                },
                // If only collection and no reqeust
                None => match &mut self.entities[collection_idx] {
                    // if by index of collection_idx collection exist - success
                    Entity::COLLECTION(_) => {
                        // if we select currently deleting entity - need to uselect
                        if self.selected_entity.collection_is_selected(collection_idx) {
                            self.selected_entity.unselect_collection();
                        }
                        self.entities.remove(collection_idx);
                        self.drop_filter();
                        self.deletion_entity.unselect_collection();
                        return true;
                    }
                    // if by indx of collection_idx reqeust exist - error
                    Entity::REQUEST(_) => return false,
                },
            },
            // If Collection not exist - prob root level
            None => match self.deletion_entity.request_idx {
                // If Request on root level
                Some(request_idx) => match self.entities[request_idx] {
                    // if on index not request, but collection - error
                    Entity::COLLECTION(_) => return false,
                    // if on index request - success
                    Entity::REQUEST(_) => {
                        // if we select currently deleting entity - need to uselect
                        if self.selected_entity.request_is_selected(None, request_idx) {
                            self.selected_entity.unselect_request();
                        }
                        self.entities.remove(request_idx);
                        self.drop_filter();
                        self.deletion_entity.unselect_request();
                        return true;
                    }
                },
                // If no collection and no request - error
                None => return false,
            },
        };
    }
}
