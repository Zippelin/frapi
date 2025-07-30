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
        self.unselect_request();
        self.collection_idx = Some(idx);
    }

    /// Uselect request
    pub fn unselect_request(&mut self) {
        self.request_idx = None;
        self.collection_idx = None;
    }

    /// Uselect collection
    #[allow(dead_code)]
    pub fn unselect_collection(&mut self) {
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
