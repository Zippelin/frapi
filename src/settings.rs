use core::fmt;
use std::{fs::File, io::BufReader};

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, Default)]
pub struct Settings {
    pub ui: UISettings,
    pub main_page: MainPageSettings,
}

impl Settings {
    pub fn load() -> Self {
        let file = File::open("cache.json").unwrap();
        let reader = BufReader::new(file);
        let result: Self = serde_json::from_reader(reader).unwrap();
        result
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, Default)]
pub struct UISettings {}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, Default)]
pub struct MainPageSettings {
    pub entities: Vec<Entity>,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub enum Entity {
    COLLECTION(CollectionSettings),
    REQUEST(RequestSettings),
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, Default)]
pub struct CollectionSettings {
    pub id: String,
    pub name: String,
    pub description: String,
    pub requests: Vec<RequestSettings>,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, Default)]
pub struct RequestSettings {
    pub id: String,
    pub name: String,
    pub protocol: Protocol,
    pub method: Method,
    pub uri: String,
    pub headers: Vec<Header>,
    pub body: String,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, Default)]
pub enum Protocol {
    HTTP,
    #[default]
    HTTPS,
    WS,
    WSS,
}

impl fmt::Display for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Protocol::HTTP => write!(f, "http"),
            Protocol::HTTPS => write!(f, "https"),
            Protocol::WS => write!(f, "ws"),
            Protocol::WSS => write!(f, "wss"),
        }
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, Default, Copy)]
#[serde(rename_all = "UPPERCASE")]
pub enum Method {
    #[default]
    GET,
    POST,
    PUT,
    PATCH,
    DELETE,
}

impl Into<reqwest::Method> for Method {
    fn into(self) -> reqwest::Method {
        match self {
            Method::GET => reqwest::Method::GET,
            Method::POST => reqwest::Method::POST,
            Method::PUT => reqwest::Method::PUT,
            Method::PATCH => reqwest::Method::PATCH,
            // TODO: в либе нет такого метода - нужно будет разобраться
            //HTTPMethod::UPDATE => Method::PATCH,
            Method::DELETE => reqwest::Method::DELETE,
        }
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, Default)]
pub struct Header {
    pub key: String,
    pub value: Value,
}

#[cfg(test)]
mod tests {
    use std::{
        fs::OpenOptions,
        io::{BufWriter, Write},
    };

    use super::*;

    #[test]
    fn generte_fake_cache() -> Result<(), std::io::Error> {
        let ui_settings = UISettings {};

        let header_1 = Header {
            key: "Host".into(),
            value: "developer.mozilla.org".into(),
        };

        let header_2 = Header {
            key: "User-Agent".into(),
            value:
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.9; rv:50.0) Gecko/20100101 Firefox/50.0"
                    .into(),
        };

        let header_3 = Header {
            key: "Upgrade-Insecure-Requests".into(),
            value: 1.into(),
        };

        let header_4 = Header {
            key: "fake-bool".into(),
            value: true.into(),
        };
        let request_1 = RequestSettings {
            id: uuid::Uuid::new_v4().to_string(),
            name: "request to yandex".into(),
            protocol: Protocol::HTTPS,
            method: Method::GET,
            uri: "ya.ru".into(),
            headers: vec![
                header_1.clone(),
                header_2.clone(),
                header_3.clone(),
                header_4.clone(),
            ],
            body: "".into(),
        };

        let request_2 = RequestSettings {
            id: uuid::Uuid::new_v4().to_string(),
            name: "request to google".into(),
            protocol: Protocol::HTTPS,
            method: Method::GET,
            uri: "google.ru".into(),
            headers: vec![
                header_1.clone(),
                header_2.clone(),
                header_3.clone(),
                header_4.clone(),
            ],
            body: "".into(),
        };

        let collection_1 = CollectionSettings {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Collection 1".into(),
            description: "Description for Collection 1".into(),
            requests: vec![request_1, request_2],
        };

        let request_1 = RequestSettings {
            id: uuid::Uuid::new_v4().to_string(),
            name: "request to microsoft".into(),
            protocol: Protocol::HTTPS,
            method: Method::GET,
            uri: "microsoft.ru".into(),
            headers: vec![
                header_1.clone(),
                header_2.clone(),
                header_3.clone(),
                header_4.clone(),
            ],
            body: "".into(),
        };

        let collection_2 = CollectionSettings {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Collection 2".into(),
            description: "Description for Collection 2".into(),
            requests: vec![request_1],
        };

        let request_1 = RequestSettings {
            id: uuid::Uuid::new_v4().to_string(),
            name: "request to mail".into(),
            protocol: Protocol::HTTPS,
            method: Method::GET,
            uri: "mail.ru".into(),
            headers: vec![
                header_1.clone(),
                header_2.clone(),
                header_3.clone(),
                header_4.clone(),
            ],
            body: "".into(),
        };

        let main_page = MainPageSettings {
            entities: vec![
                Entity::COLLECTION(collection_1),
                Entity::COLLECTION(collection_2),
                Entity::REQUEST(request_1),
            ],
        };

        let application = Settings {
            ui: ui_settings,
            main_page,
        };

        let json = serde_json::to_string_pretty(&application).unwrap();

        println!("{}", json);

        let file = OpenOptions::new()
            .append(false)
            .create(true)
            .write(true)
            .open("/Users/denis/rustProjects/frapi_v2/cache.json")
            .unwrap();

        let mut buffer = BufWriter::new(&file);
        let _ = buffer.write_all(json.as_bytes());
        Ok(())
    }
}
