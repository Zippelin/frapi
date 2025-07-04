use crate::{
    settings::{Settings, UISettings},
    states::main_page::MainPage,
};

pub mod main_page;

pub struct Pages {
    pub current: Page,
    pub main_page: MainPage,
}

impl From<&Settings> for Pages {
    fn from(value: &Settings) -> Self {
        Self {
            current: Page::MAIN,
            main_page: MainPage::from(value),
        }
    }
}

#[derive(Debug)]
pub struct Style {}

impl From<&UISettings> for Style {
    fn from(value: &UISettings) -> Self {
        Self {}
    }
}

pub enum Page {
    MAIN,
}
