use chrono::{DateTime, Local};
use serde_json::Value;
use tokio_tungstenite::tungstenite::Utf8Bytes;

use crate::states::main_page::{generics::Header, response::response_data::ResponseData};
use reqwest::{Error, Response as HttpResponse};

pub mod response_data;

/// Reqeust response state representation
#[derive(Debug, Clone)]
pub struct Response {
    /// local time response received
    pub time: DateTime<Local>,
    /// response data details
    pub data: ResponseData,
    /// representation of selected view for UI
    pub selected_view: ResponseView,
    /// HTTP code
    pub code: usize,
    /// state of UI element of folded
    pub is_folded: bool,
    // TODO: add how long request took. Troubles how to save request send time.
}

impl Response {
    /// Used as answear from WSS ot WS
    pub fn from_utf8_bytes(data: Utf8Bytes) -> Self {
        Self {
            time: Local::now(),
            data: ResponseData::new(data.to_string(), vec![], "".into()),
            selected_view: ResponseView::RAW,
            code: 0,
            is_folded: true,
        }
    }

    pub fn closed_connection() -> Self {
        Self {
            time: Local::now(),
            data: ResponseData::new("Connection closed".into(), vec![], "".into()),
            selected_view: ResponseView::RAW,
            code: 0,
            is_folded: true,
        }
    }

    /// Used as answear from HTTP or HTTPS
    pub async fn from_http_response(http_response: HttpResponse) -> Result<Self, (Self, String)> {
        match http_response.error_for_status() {
            Ok(mut result) => {
                let mut headers = vec![];

                for header in result.headers() {
                    headers.push(Header {
                        key: header.0.to_string(),
                        value: header.1.to_str().unwrap().to_string(),
                    });
                }

                let code = result.status().as_u16() as usize;
                let redirect_url = result.url().to_string();

                while let Ok(Some(chunk)) = result.chunk().await {
                    println!("Chunk: {chunk:?}");
                }

                match result.text().await {
                    Ok(text) => {
                        println!("text: {}", text);

                        Ok(Self {
                            time: Local::now(),
                            data: ResponseData::new(text, headers, redirect_url),
                            selected_view: ResponseView::RAW,
                            code,
                            is_folded: true,
                        })
                    }
                    Err(err) => {
                        println!("err: {}", err);
                        Err((
                            Self {
                                time: Local::now(),
                                data: ResponseData::new(
                                    "Error during text receiving".into(),
                                    vec![],
                                    "".into(),
                                ),
                                selected_view: ResponseView::RAW,
                                code,
                                is_folded: true,
                            },
                            format!("Error during text receiving. Error: {}", err),
                        ))
                    }
                }
            }

            Err(err) => match err.status() {
                Some(err_status_code) => match err_status_code.canonical_reason() {
                    Some(err_reason) => Ok(Self {
                        time: Local::now(),
                        data: ResponseData::new(err_reason.into(), vec![], "".into()),
                        selected_view: ResponseView::RAW,
                        code: err.status().unwrap().as_u16() as usize,
                        is_folded: true,
                    }),
                    None => Err((
                        Self {
                            time: Local::now(),
                            data: ResponseData::new(
                                "Error during text receiving for error reason".into(),
                                vec![],
                                "".into(),
                            ),
                            selected_view: ResponseView::RAW,
                            code: err.status().unwrap().as_u16() as usize,
                            is_folded: true,
                        },
                        "Error during text receiving for error reason".into(),
                    )),
                },
                None => Err((
                    Self {
                        time: Local::now(),
                        data: ResponseData::new(
                            "Error during status code receiving".into(),
                            vec![],
                            "".into(),
                        ),
                        selected_view: ResponseView::RAW,
                        code: err.status().unwrap().as_u16() as usize,
                        is_folded: true,
                    },
                    "Error during text receiving".into(),
                )),
            },
        }
    }

    /// Used as answear from HTTP or HTTPS for HTTP errors
    pub async fn from_http_error(error: Error) -> Result<Self, (Self, String)> {
        let code = match error.status() {
            Some(val) => val.as_u16() as usize,
            None => 0,
        };

        match error.status() {
            Some(status_code) => match status_code.canonical_reason() {
                Some(canonical_reason) => Ok(Self {
                    time: Local::now(),
                    data: ResponseData::new(canonical_reason.into(), vec![], "".into()),
                    selected_view: ResponseView::RAW,
                    code,
                    is_folded: true,
                }),
                None => Err((
                    Self {
                        time: Local::now(),
                        data: ResponseData::new(
                            "During Request Error occured. Could not read error status code."
                                .into(),
                            vec![],
                            "".into(),
                        ),
                        selected_view: ResponseView::RAW,
                        code,
                        is_folded: true,
                    },
                    "During Request Error occured. Could not read error status code.".into(),
                )),
            },
            None => Err((
                Self {
                    time: Local::now(),
                    data: ResponseData::new(
                        "During Request Error occured. Could not read error reason.".into(),
                        vec![],
                        "".into(),
                    ),
                    selected_view: ResponseView::RAW,
                    code,
                    is_folded: true,
                },
                "During Request Error occured. Could not read error reason.".into(),
            )),
        }
    }
}

/// Json representations for UI
#[derive(Debug, Clone, PartialEq)]
pub enum JsonViewType {
    Simple,
    Comlex,
}

/// Json data
#[derive(Debug, Clone)]
pub struct JsonView {
    pub simple: String,
    pub complex: Value,
    pub view_type: JsonViewType,
}

impl JsonView {
    pub fn new(text: &String) -> Self {
        let (simple, complex) = match serde_json::from_str(text) {
            Ok(val) => {
                let beauty = serde_json::to_string_pretty(&val);
                if beauty.is_err() {
                    (text.clone(), val)
                } else {
                    (beauty.unwrap(), val)
                }
            }
            Err(_) => ("".into(), Value::Null),
        };
        Self {
            simple,
            complex,
            view_type: JsonViewType::Simple,
        }
    }
}

/// Response representations types
#[derive(Debug, Clone, PartialEq)]
pub enum ResponseView {
    JSON,
    RAW,
    HEADERS,
}
