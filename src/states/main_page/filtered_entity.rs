use crate::states::main_page::entity::Entity;

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
