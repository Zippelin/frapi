use serde::{Deserialize, Serialize};

use crate::{settings::main_settings::entity::Entity, states::main_page::MainPage};

pub mod entity;

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, Default)]
pub struct MainPageSettings {
    pub entities: Vec<Entity>,
}

impl From<&MainPage> for MainPageSettings {
    fn from(value: &MainPage) -> Self {
        let mut entities = vec![];
        for state_entity in &value.entities {
            entities.push(Entity::from(state_entity));
        }
        Self { entities }
    }
}

impl MainPageSettings {
    pub fn default() -> Self {
        Self { entities: vec![] }
    }
    pub fn from_original(value: &MainPage) -> Self {
        let mut entities = vec![];
        for state_entity in &value.entities {
            entities.push(Entity::from_original(state_entity));
        }
        Self { entities }
    }
}
