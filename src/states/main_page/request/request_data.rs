use crate::{
    settings::{Method, Protocol, RequestSettings},
    states::main_page::generics::{CountedText, Header},
};

/// Request data representation
#[derive(Debug, Clone)]
pub struct RequestData {
    pub name: String,
    pub protocol: Protocol,
    pub method: Method,
    pub uri: String,
    pub headers: Vec<Header>,
    pub body: CountedText,
    pub message: CountedText,
    /// List og qeury params.
    /// Constructed on fly, dont store in settings
    pub query_params: Vec<Header>,
    /// Path to binary file
    pub binary_path: String,
}

/// From Settigns -> State
impl From<&RequestSettings> for RequestData {
    fn from(value: &RequestSettings) -> Self {
        let mut headers = vec![];

        for header in &value.headers {
            // Remove redundant quotes from only string-like values
            match header.value.as_str() {
                Some(val) => {
                    headers.push(Header {
                        key: header.key.clone(),
                        value: val.to_string(),
                    });
                }
                None => {
                    headers.push(Header {
                        key: header.key.clone(),
                        value: header.value.to_string(),
                    });
                }
            };
        }

        let mut body = CountedText::default();
        body.set(value.body.clone());

        let mut message = CountedText::default();
        message.set(value.message.clone());

        let mut data = Self {
            name: value.name.clone(),
            protocol: value.protocol.clone(),
            method: value.method.clone(),
            uri: value.uri.clone(),
            headers,
            body,
            query_params: vec![],
            message,
            binary_path: value.binary_path.clone(),
        };
        data.parse_query_params();
        data
    }
}

impl RequestData {
    pub fn default() -> Self {
        Self {
            name: "New Request".into(),
            protocol: Protocol::HTTP,
            method: Method::GET,
            uri: "".into(),
            headers: vec![],
            body: CountedText::default(),
            query_params: vec![],
            message: CountedText::default(),
            binary_path: "".into(),
        }
    }
    /// Copy from other Self.
    /// Usualy used on Save.
    pub fn copy_from_other(&mut self, other_request: &Self) {
        self.name = other_request.name.clone();
        self.protocol = other_request.protocol.clone();
        self.method = other_request.method.clone();
        self.uri = other_request.uri.clone();
        self.body = other_request.body.clone();
        self.message = other_request.message.clone();

        self.headers = other_request
            .headers
            .iter()
            .map(|val| Header {
                key: val.key.clone(),
                value: val.value.clone(),
            })
            .collect();
    }

    /// Parse Url string
    pub fn parse_url(&mut self) {
        self.parse_url_protocol();
        self.parse_query_params();
    }

    // Parsing protocol of URL and trim it
    fn parse_url_protocol(&mut self) {
        let url = self.uri.to_lowercase();
        let mut split_url = if url.starts_with("http:") {
            self.protocol = Protocol::HTTP;
            url.split_at(5).1.to_string()
        } else if url.starts_with("https:") {
            self.protocol = Protocol::HTTPS;
            url.split_at(6).1.to_string()
        } else if url.starts_with("ws:") {
            self.protocol = Protocol::WS;
            url.split_at(3).1.to_string()
        } else if url.starts_with("wss:") {
            self.protocol = Protocol::WSS;
            url.split_at(4).1.to_string()
        } else {
            self.protocol = Protocol::HTTPS;
            url
        };

        while split_url.starts_with("\\") || split_url.starts_with("/") {
            split_url = split_url.split_at(1).1.to_string();
        }
        self.uri = split_url;
    }

    /// Parse from URL QeryParams
    fn parse_query_params(&mut self) {
        let url = self.uri.clone();
        let url_parts = url.split_once("?");

        let mut query_params_vec: Vec<Header> = vec![];
        if let Some((_, query_string_part)) = url_parts {
            let query_strings = query_string_part.split("&");
            for part in query_strings {
                let part_split = part.split_once("=");

                if let Some((part_key, part_value)) = part_split {
                    query_params_vec.push(Header {
                        key: part_key.to_string(),
                        value: part_value.to_string(),
                    });
                } else {
                    query_params_vec.push(Header {
                        key: part.to_string(),
                        value: "".into(),
                    });
                }
            }

            self.query_params = query_params_vec;
        };
    }

    /// Constructing URL from base url and QueryParams
    pub fn contruct_url(&mut self) {
        if self.query_params.len() == 0 {
            self.uri = self
                .uri
                .split("?")
                .next()
                .unwrap_or(self.uri.as_str())
                .into();
            return;
        };

        let base_url = self.uri.split("?").next().unwrap_or(self.uri.as_str());

        let mut query_params_string = "".to_string();
        for i in 0..self.query_params.len() {
            query_params_string = format!(
                "{}{}={}",
                query_params_string, self.query_params[i].key, self.query_params[i].value
            );

            if i < self.query_params.len() - 1 {
                query_params_string += "&";
            }
        }

        self.uri = format!("{}?{}", base_url, query_params_string)
    }

    pub fn protocot_is_ws(&self) -> bool {
        if [Protocol::WS, Protocol::WSS].contains(&self.protocol) {
            return true;
        }
        false
    }

    pub fn protocot_is_http(&self) -> bool {
        if [Protocol::HTTP, Protocol::HTTPS].contains(&self.protocol) {
            return true;
        }
        false
    }
}
