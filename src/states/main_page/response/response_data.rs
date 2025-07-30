use crate::states::main_page::{generics::Header, response::JsonView};

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
