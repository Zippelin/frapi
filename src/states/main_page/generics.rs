// TODO: add enabled o disabled status of header - also need changes in settings
/// Header of Request State
#[derive(Debug, Clone)]
pub struct Header {
    pub key: String,
    pub value: String,
}

impl Header {
    pub fn default() -> Self {
        Self {
            key: "".into(),
            value: "".into(),
        }
    }
}

/// Message to send via WS
#[derive(Debug, Clone)]
pub struct CountedText {
    pub message: String,
    pub rows: usize,
}

impl CountedText {
    pub fn set(&mut self, message: String) {
        self.message = message;
        self.update_rows();
    }

    pub fn update_rows(&mut self) {
        let splitted: Vec<&str> = self.message.split("\n").collect();
        self.rows = splitted.len();
    }

    pub fn default() -> Self {
        Self {
            message: "".into(),
            rows: 0,
        }
    }
}
