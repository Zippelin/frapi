use std::fmt::Debug;

use egui::{FontFamily, FontId, FontSelection, RichText};

#[derive(Debug, Clone)]
pub struct Fonts {
    big_textedit: FontId,
    medium_textedit: FontId,
    small_textedit: FontId,
    header1: FontId,
    header2: FontId,
    label_strong: FontId,
}
// TODO: Add this fonts support across application
/// Fonts settings for application
impl Fonts {
    pub fn new() -> Self {
        Self {
            big_textedit: Self::fontid_textedit_big(),
            medium_textedit: Self::fontid_textedit_medium(),
            small_textedit: Self::fontid_textedit_small(),
            header1: Self::fontid_header1(),
            header2: Self::fontid_header2(),
            label_strong: Self::fontid_label_strong(),
        }
    }

    fn fontid_label_strong() -> FontId {
        FontId {
            size: 12.,
            family: FontFamily::Monospace,
        }
    }

    fn fontid_textedit_big() -> FontId {
        FontId {
            size: 16.,
            family: FontFamily::Monospace,
        }
    }

    fn fontid_textedit_medium() -> FontId {
        FontId {
            size: 14.,
            family: FontFamily::Monospace,
        }
    }
    fn fontid_textedit_small() -> FontId {
        FontId {
            size: 12.,
            family: FontFamily::Monospace,
        }
    }

    fn fontid_header1() -> FontId {
        FontId::new(30.0, FontFamily::Monospace)
    }
    fn fontid_header2() -> FontId {
        FontId::new(15.0, FontFamily::Monospace)
    }

    pub fn textedit_big(&self) -> FontSelection {
        FontSelection::FontId(self.big_textedit.clone())
    }
    pub fn textedit_medium(&self) -> FontSelection {
        FontSelection::FontId(self.medium_textedit.clone())
    }
    pub fn textedit_small(&self) -> FontSelection {
        FontSelection::FontId(self.small_textedit.clone())
    }

    pub fn menu_text(&self, text: &str) -> RichText {
        RichText::new(text).monospace().strong()
    }

    pub fn label_text(&self, text: &str) -> RichText {
        RichText::new(text).monospace().strong().size(14.)
    }

    pub fn header1(&self) -> FontId {
        self.header1.clone()
    }

    pub fn header2(&self) -> FontId {
        self.header2.clone()
    }

    pub fn label_strong(&self) -> FontId {
        self.label_strong.clone()
    }
}
