/// Stated of Main page of application
///
use crate::{
    executor::State,
    settings::Settings,
    states::{
        main_page::{
            collection::Collection, entity::Entity, filtered_entity::FilteredEntities,
            request::Request, selected_entity::SelectedEntity,
        },
        Style,
    },
};

pub mod collection;
pub mod entity;
pub mod filtered_entity;
pub mod generics;
pub mod request;
pub mod response;
pub mod selected_entity;

/// Types of right panel
#[derive(Debug, Clone, PartialEq)]
pub enum RightPanelType {
    EVENTS,
}

/// State of right panel bassed on user interraction
#[derive(Debug, Clone)]
pub struct RightPanel {
    pub is_visible: bool,
    pub panel_type: RightPanelType,
}

impl Default for RightPanel {
    fn default() -> Self {
        Self {
            is_visible: false,
            panel_type: RightPanelType::EVENTS,
        }
    }
}

impl RightPanel {
    pub fn toggle(&mut self) {
        self.is_visible = !self.is_visible;
    }

    pub fn toggle_events(&mut self) {
        if self.panel_type == RightPanelType::EVENTS {
            self.is_visible = !self.is_visible;
        } else {
            self.panel_type = RightPanelType::EVENTS;
        }
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
    /// To mark for deletion selet use `select_request` or `select_collection`
    pub deletion_entity: SelectedEntity,
    /// Filter string
    pub filter_text: String,
    /// Description where to move entity
    pub request_move_target: RequestMoveTarget,
    /// Styles and colors for UI theme
    pub style: Style,
    /// Visible or not right sided panel
    pub right_panel: RightPanel,
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
            right_panel: RightPanel::default(),
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

    /// Drop filter and regenerate idexes to display
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

    pub fn get_request_mut(&mut self, idx: usize) -> Option<&mut Request> {
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

    /// Get len of selected inner items (For Collections - amount of reqeusts)
    pub fn get_selected_items_count(&self) -> usize {
        if !self.selected_entity.is_selected() {
            return self.entities.len();
        };

        if self.selected_request().is_some() {
            return 1;
        };

        if let Some(collection) = self.selected_collection() {
            return collection.requests.len();
        };

        return self.entities.len();
    }

    /// Test state of selected reqquest executor
    pub fn selected_request_executor_state(&self) -> String {
        if let Some(request) = self.selected_request() {
            let state = request.executor.state.lock();
            let state = state.unwrap().clone();
            match state {
                State::FREE => return "FREE".into(),
                State::BUSY => return "BUSY".into(),
                State::CONNECTED => return "CONNECTED".into(),
            }
        }
        return "FREE".into();
    }
}
