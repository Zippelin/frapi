use crate::{
    settings::main_settings::entity::collection_settings::CollectionSettings,
    states::main_page::request::Request,
};

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
