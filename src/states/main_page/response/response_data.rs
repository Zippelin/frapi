use crate::states::main_page::{generics::Header, response::JsonView};

/// Response data state
#[derive(Debug, Clone)]
pub struct ResponseData {
    pub raw: String,
    pub json: JsonView,
    pub headers: Vec<Header>,
    pub redictection_url: String,
    pub redirection: Vec<Redicrections>,
}

impl ResponseData {
    pub fn new(raw: String, headers: Vec<Header>, redictection_url: String) -> Self {
        Self {
            json: JsonView::new(&raw),
            raw,
            headers,
            redictection_url,
            redirection: vec![],
        }
    }

    /// Checking if JSON state exist for raw response data
    pub fn json_is_exist(&self) -> bool {
        !self.json.complex.is_null()
    }
    
}

/// Data with page redirection after initial request
#[derive(Debug, Clone)]
pub struct Redicrections {
    pub headers: Vec<Header>,
    pub redictection_url: String,
}
