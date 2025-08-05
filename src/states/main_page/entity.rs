use crate::{
    settings::main_settings::entity::Entity as SettingsEntity,
    states::main_page::{collection::Collection, request::Request},
};

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
