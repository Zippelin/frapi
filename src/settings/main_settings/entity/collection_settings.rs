use serde::{Deserialize, Serialize};

use crate::{
    settings::main_settings::entity::request_settings::RequestSettings,
    states::main_page::collection::Collection as StateCollection,
};

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, Default)]
pub struct CollectionSettings {
    pub id: String,
    pub name: String,
    pub description: String,
    pub requests: Vec<RequestSettings>,
}

impl From<&StateCollection> for CollectionSettings {
    fn from(value: &StateCollection) -> Self {
        let requests = value
            .requests
            .iter()
            .map(|val| RequestSettings::from(val))
            .collect();
        Self {
            id: value.id.clone(),
            name: value.draft.name.clone(),
            description: value.draft.description.clone(),
            requests,
        }
    }
}

impl CollectionSettings {
    pub fn from_original(value: &StateCollection) -> Self {
        let requests = value
            .requests
            .iter()
            .map(|val| RequestSettings::from(val))
            .collect();
        Self {
            id: value.id.clone(),
            name: value.original.name.clone(),
            description: value.original.description.clone(),
            requests,
        }
    }
}
