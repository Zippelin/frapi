use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::states::main_page::request::request_data::{BodyFromData, FormFieldType, RequestBody};

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct RequestBodySettigns {
    pub raw: String,
    pub form_data: Vec<BodyFromDataSettings>,
    pub binary_path: String,
}

impl From<&RequestBody> for RequestBodySettigns {
    fn from(value: &RequestBody) -> Self {
        let form_data = value
            .form_data
            .iter()
            .map(|val| BodyFromDataSettings::from(val))
            .collect();
        Self {
            raw: value.raw.message.clone(),
            form_data: form_data,
            binary_path: value.binary_path.clone(),
        }
    }
}

impl Default for RequestBodySettigns {
    fn default() -> Self {
        Self {
            raw: "".into(),
            form_data: vec![],
            binary_path: "".into(),
        }
    }
}

/// Request Form Body data Fied Type
#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, Default)]
pub enum FormFieldTypeSettings {
    #[default]
    Text,
    File,
}

/// Request Form Body data
#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct BodyFromDataSettings {
    pub key: String,
    pub value: Value,
    pub field_type: FormFieldTypeSettings,
}

impl From<&BodyFromData> for BodyFromDataSettings {
    fn from(value: &BodyFromData) -> Self {
        Self {
            key: value.key.clone(),
            value: Value::from(value.value.clone()),
            field_type: match value.field_type {
                FormFieldType::Text => FormFieldTypeSettings::Text,
                FormFieldType::File => FormFieldTypeSettings::File,
            },
        }
    }
}
