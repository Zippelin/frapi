use core::fmt;
use std::{
    fs::{File, OpenOptions},
    io::{BufReader, Write},
    path::PathBuf,
};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::states::{
    main_page::{
        Collection as StateCollection, Entity as StateEntity, Header as StateHeader, MainPage,
        Request as StateRequest,
    },
    Options as StateOptions, States, Style, Theme,
};

fn default_settings_filepath() -> String {
    "cache.json".into()
}

// All settings from application
#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, Default)]
pub struct Settings {
    pub ui: UISettings,
    pub main_page: MainPageSettings,
    pub options: Options,
}

impl Settings {
    /// Initial load of by default path
    pub fn load() -> Self {
        let file = match File::open(default_settings_filepath()) {
            Ok(val) => val,
            Err(err) => match err.kind() {
                std::io::ErrorKind::NotFound => {
                    let default_settings = Self::default();
                    let settings_string = match serde_json::to_string(&default_settings) {
                        Ok(val) => val,
                        Err(err) => {
                            panic!(
                                "Error: could not create default settings string.\n
                                Trace: {}",
                                err
                            );
                        }
                    };

                    match OpenOptions::new()
                        .create(true)
                        .write(true)
                        .open(default_settings_filepath())
                    {
                        Ok(mut val) => match val.write_all(settings_string.as_bytes()) {
                            Ok(_) => {}
                            Err(err) => {
                                panic!(
                                    "Error: could not save default settings file. Path: {}\n
                                    Trace: {}",
                                    default_settings_filepath(),
                                    err
                                );
                            }
                        },
                        Err(err) => {
                            panic!(
                                "Error: could not create default settings file. Path: {}\n
                                Trace: {}",
                                default_settings_filepath(),
                                err
                            );
                        }
                    };

                    return default_settings;
                }

                other_error => {
                    panic!(
                        "Error: Could not load settings file. Path: {}\n
                        Trace: {}",
                        default_settings_filepath(),
                        other_error
                    );
                }
            },
        };
        let reader = BufReader::new(file);
        let result = serde_json::from_reader(reader);
        match result {
            Ok(val) => val,
            Err(err) => {
                panic!(
                    "Error: Could not parse settings file. Path: {}\n
                    Please fix it, or delete so application can recreate default one.\n
                    Be aware that in case of deletion you will lose all save reqeusts adn collections.\n
                    Trace: {}",
                    default_settings_filepath(),
                    err
                );
            }
        }
    }

    /// Load settings during work with events
    pub fn dyn_load(file_path: Option<PathBuf>) -> Result<Self, String> {
        let path = match file_path {
            Some(val) => val.to_str().unwrap().to_string(),
            None => default_settings_filepath(),
        };
        let file = match File::open(path) {
            Ok(val) => val,
            Err(err) => {
                return Err(format!("Could not load settings file. Error: {err}"));
            }
        };
        let reader = BufReader::new(file);

        match serde_json::from_reader(reader) {
            Ok(val) => Ok(val),
            Err(err) => {
                return Err(format!("Could not parse settings file. Trace: {err}"));
            }
        }
    }

    fn default() -> Self {
        Self {
            ui: UISettings::default(),
            main_page: MainPageSettings::default(),
            options: Options::default(),
        }
    }

    pub fn save(&self, save_to_path: Option<PathBuf>) -> Result<(), String> {
        let settings_string = match serde_json::to_string_pretty(self) {
            Ok(val) => val,
            Err(err) => {
                return Err(format!(
                    "Error: Could not convert settings to strings.\nTrace: {err}"
                ))
            }
        };

        let path = if save_to_path.is_none() {
            default_settings_filepath()
        } else {
            save_to_path.unwrap().to_str().unwrap().to_string()
        };

        match OpenOptions::new()
            .write(true)
            .append(false)
            .truncate(true)
            .create(true)
            .open(&path)
        {
            Ok(mut file) => {
                match file.write(settings_string.as_bytes()) {
                    Ok(_) => {}
                    Err(err) => {
                        return Err(format!(
                            "Error: Could not save settings file. Path: {}.\nTrace: {err}",
                            &path
                        ));
                    }
                };
            }
            Err(err) => {
                return Err(format!(
                    "Error: Could not open settings file. Path: {}.\nTrace: {err}",
                    &path
                ))
            }
        };
        Ok(())
    }
}

impl From<&States> for Settings {
    fn from(value: &States) -> Self {
        Self {
            ui: UISettings::from(&value.style),
            main_page: MainPageSettings::from(&value.main_page),
            options: Options::from(&value.options),
        }
    }
}

// Settings::Options - general settings
#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, Default)]
pub struct Options {}

impl From<&StateOptions> for Options {
    fn from(value: &StateOptions) -> Self {
        let _ = value;
        // TODO: add convertions from State-Optiont to Settigns Options
        Self {}
    }
}

impl Options {
    pub fn default() -> Self {
        Self {}
    }
}

// Settings::Theme - color theme for application
#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, Default)]
pub enum UITheme {
    Light,
    #[default]
    Dark,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, Default)]
pub struct UISettings {
    pub theme: UITheme,
}

impl From<&Style> for UISettings {
    fn from(value: &Style) -> Self {
        let theme = match &value.theme {
            Theme::Light(_) => UITheme::Light,
            Theme::Dark(_) => UITheme::Dark,
        };
        Self { theme }
    }
}

impl UISettings {
    pub fn default() -> Self {
        Self {
            theme: UITheme::Dark,
        }
    }
}

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
}

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

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, Default)]
pub struct CollectionSettings {
    pub id: String,
    pub name: String,
    pub description: String,
    pub requests: Vec<RequestSettings>,
}

impl From<&StateCollection> for CollectionSettings {
    fn from(value: &StateCollection) -> Self {
        let requests = value
            .requests
            .iter()
            .map(|val| RequestSettings::from(val))
            .collect();
        Self {
            id: value.id.clone(),
            name: value.draft.name.clone(),
            description: value.draft.description.clone(),
            requests,
        }
    }
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

impl From<&StateRequest> for RequestSettings {
    fn from(value: &StateRequest) -> Self {
        let headers = value
            .draft
            .headers
            .iter()
            .map(|val| Header::from(val))
            .collect();
        Self {
            id: value.id.clone(),
            name: value.draft.name.clone(),
            protocol: value.draft.protocol.clone(),
            method: value.draft.method.clone(),
            uri: value.draft.uri.clone(),
            headers,
            body: value.draft.body.clone(),
        }
    }
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
            Protocol::HTTP => write!(f, "HTTP"),
            Protocol::HTTPS => write!(f, "HTTPS"),
            Protocol::WS => write!(f, "WS"),
            Protocol::WSS => write!(f, "WSS"),
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

impl fmt::Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Method::GET => write!(f, "GET"),
            Method::POST => write!(f, "POST"),
            Method::PUT => write!(f, "PUT"),
            Method::PATCH => write!(f, "PATCH"),
            Method::DELETE => write!(f, "DELETE"),
        }
    }
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

impl From<&StateHeader> for Header {
    fn from(value: &StateHeader) -> Self {
        match serde_json::from_str(&value.value) {
            Ok(val) => Self {
                key: value.key.clone(),
                value: val,
            },
            Err(_) => Self {
                key: value.key.clone(),
                value: Value::from(value.value.clone()),
            },
        }
    }
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
        let ui_settings = UISettings {
            theme: UITheme::Dark,
        };

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
        let request_0 = RequestSettings {
            id: uuid::Uuid::new_v4().to_string(),
            name: "request to firebase".into(),
            protocol: Protocol::HTTPS,
            method: Method::GET,
            uri: "vue-backend-mocker-default-rtdb.europe-west1.firebasedatabase.app/surveys.json"
                .into(),
            headers: vec![
                header_1.clone(),
                header_2.clone(),
                header_3.clone(),
                header_4.clone(),
            ],
            body: "".into(),
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
                Entity::REQUEST(request_0),
                Entity::COLLECTION(collection_1),
                Entity::COLLECTION(collection_2),
                Entity::REQUEST(request_1),
            ],
        };

        let options = Options {};

        let application = Settings {
            ui: ui_settings,
            main_page,
            options,
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
