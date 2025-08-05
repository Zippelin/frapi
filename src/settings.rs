use std::{
    fs::{File, OpenOptions},
    io::{BufReader, Write},
    path::PathBuf,
};

use serde::{Deserialize, Serialize};

use crate::{
    settings::{
        main_settings::MainPageSettings, options_settings::OptionsSettings, ui_settings::UISettings,
    },
    states::{
        main_page::request::{HttpVersion, RequestHttpSetup, RequestWsSetup},
        States,
    },
};

pub mod main_settings;
pub mod options_settings;
pub mod ui_settings;

fn default_settings_filepath() -> String {
    "cache.json".into()
}

// All settings from application
#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, Default)]
pub struct Settings {
    pub ui: UISettings,
    pub main_page: MainPageSettings,
    pub options: OptionsSettings,
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
            options: OptionsSettings::default(),
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

    /// From States -> Settings.
    /// All changes took from "original" fields.
    /// Used as part of Save On New Entity
    pub fn from_original(value: &States) -> Self {
        Self {
            ui: UISettings::from(&value.style),
            main_page: MainPageSettings::from_original(&value.main_page),
            options: OptionsSettings::from(&value.options),
        }
    }
}

impl From<&States> for Settings {
    /// From States -> Settings.
    /// All changes took from "draft" fields.
    /// Used as part of Save All
    fn from(value: &States) -> Self {
        Self {
            ui: UISettings::from(&value.style),
            main_page: MainPageSettings::from(&value.main_page),
            options: OptionsSettings::from(&value.options),
        }
    }
}

/// Generate chache based on currend model
#[cfg(test)]
mod tests {
    use std::{
        fs::OpenOptions,
        io::{BufWriter, Write},
    };

    use crate::settings::{
        main_settings::entity::{
            collection_settings::CollectionSettings,
            request_settings::{
                body_settings::RequestBodySettigns, method_settigns::Method,
                protocol_settings::Protocol, request_setup_settings::RequestSetupSettings, Header,
                RequestSettings,
            },
            Entity,
        },
        ui_settings::UITheme,
    };

    use super::*;

    #[test]
    fn generte_fake_cache() -> Result<(), std::io::Error> {
        let ui_settings = UISettings {
            theme: UITheme::Dark,
        };

        // let header_1 = Header {
        //     key: "Host".into(),
        //     value: "developer.mozilla.org".into(),
        // };

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
                // header_1.clone(),
                header_2.clone(),
                header_3.clone(),
                header_4.clone(),
            ],
            body: RequestBodySettigns::default(),
            message: "".into(),
            setup: RequestSetupSettings::http(),
        };

        let request_1 = RequestSettings {
            id: uuid::Uuid::new_v4().to_string(),
            name: "request to yandex".into(),
            protocol: Protocol::HTTPS,
            method: Method::GET,
            uri: "ya.ru".into(),
            headers: vec![
                // header_1.clone(),
                header_2.clone(),
                header_3.clone(),
                header_4.clone(),
            ],
            body: RequestBodySettigns::default(),
            message: "".into(),
            setup: RequestSetupSettings::http(),
        };

        let request_2 = RequestSettings {
            id: uuid::Uuid::new_v4().to_string(),
            name: "request to google".into(),
            protocol: Protocol::HTTPS,
            method: Method::GET,
            uri: "google.ru".into(),
            headers: vec![
                // header_1.clone(),
                header_2.clone(),
                header_3.clone(),
                header_4.clone(),
            ],
            body: RequestBodySettigns::default(),
            message: "".into(),
            setup: RequestSetupSettings::http(),
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
                // header_1.clone(),
                header_2.clone(),
                header_3.clone(),
                header_4.clone(),
            ],
            body: RequestBodySettigns::default(),
            message: "".into(),
            setup: RequestSetupSettings::http(),
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
                // header_1.clone(),
                header_2.clone(),
                header_3.clone(),
                header_4.clone(),
            ],
            body: RequestBodySettigns::default(),
            message: "".into(),
            setup: RequestSetupSettings::http(),
        };

        let main_page = MainPageSettings {
            entities: vec![
                Entity::REQUEST(request_0),
                Entity::COLLECTION(collection_1),
                Entity::COLLECTION(collection_2),
                Entity::REQUEST(request_1),
            ],
        };

        let options = OptionsSettings {
            window_size: (800., 600.),
            window_position: None,
        };

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
            .truncate(true)
            .open(default_settings_filepath())
            .unwrap();

        let mut buffer = BufWriter::new(&file);
        let _ = buffer.write_all(json.as_bytes());
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum HttpVersionSetting {
    #[default]
    AUTO,
    HTTPv1,
    HTTPv2,
}

/// Settings to make http reqeust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RequestHttpSetupSettings {
    pub http_version: HttpVersionSetting,
    pub use_cookies: bool,
    pub use_redirects: bool,
    pub redirects_amount: usize,
}

impl Default for RequestHttpSetupSettings {
    fn default() -> Self {
        Self {
            http_version: HttpVersionSetting::AUTO,
            use_cookies: true,
            use_redirects: true,
            redirects_amount: 9,
        }
    }
}

impl From<&RequestHttpSetup> for RequestHttpSetupSettings {
    fn from(value: &RequestHttpSetup) -> Self {
        let http_version = match value.http_version {
            HttpVersion::HTTPv1 => HttpVersionSetting::HTTPv1,
            HttpVersion::HTTPv2 => HttpVersionSetting::HTTPv2,
            HttpVersion::AUTO => HttpVersionSetting::AUTO,
        };
        Self {
            http_version,
            use_cookies: value.use_cookies,
            use_redirects: value.use_redirects,
            redirects_amount: value.redirects_amount.parse::<usize>().unwrap(),
        }
    }
}

/// Settings to make ws reqeust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RequestWsSetupSettings {
    pub reconnection_timeout: usize,
    pub reconnection_attempts: usize,
}

impl Default for RequestWsSetupSettings {
    fn default() -> Self {
        Self {
            reconnection_timeout: 5000,
            reconnection_attempts: 3,
        }
    }
}

impl From<&RequestWsSetup> for RequestWsSetupSettings {
    fn from(value: &RequestWsSetup) -> Self {
        Self {
            reconnection_timeout: value.reconnection_timeout.parse::<usize>().unwrap(),
            reconnection_attempts: value.reconnection_attempts.parse::<usize>().unwrap(),
        }
    }
}
