use serde::{Deserialize, Serialize};

use crate::{
    settings::main_settings::entity::{
        collection_settings::CollectionSettings, request_settings::RequestSettings,
    },
    states::main_page::entity::Entity as StateEntity,
};

pub mod collection_settings;
pub mod request_settings;

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub enum Entity {
    COLLECTION(CollectionSettings),
    REQUEST(RequestSettings),
}

impl From<&StateEntity> for Entity {
    fn from(value: &StateEntity) -> Self {
        match value {
            StateEntity::COLLECTION(collection) => {
                Self::COLLECTION(CollectionSettings::from(collection))
            }
            StateEntity::REQUEST(request) => Self::REQUEST(RequestSettings::from(request)),
        }
    }
}

impl Entity {
    pub fn from_original(value: &StateEntity) -> Self {
        match value {
            StateEntity::COLLECTION(collection) => {
                Self::COLLECTION(CollectionSettings::from_original(collection))
            }
            StateEntity::REQUEST(request) => Self::REQUEST(RequestSettings::from_original(request)),
        }
    }
}
