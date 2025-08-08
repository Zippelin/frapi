use egui::Color32;

#[derive(Debug, Clone)]
pub struct ThemeColors {
    pub main: Color32,
    pub secondary: Color32,
    pub danger: Color32,
    pub light: Color32,
    pub lighter: Color32,
    pub success: Color32,
    pub warning: Color32,
}

// TODO: make more colors for various widgets
impl ThemeColors {
    pub fn dark() -> Self {
        Self {
            main: Color32::from_hex("#212529").unwrap(),
            secondary: Color32::from_hex("#6c757d").unwrap(),
            danger: Color32::from_hex("#ee6c4d").unwrap(),
            light: Color32::from_hex("#848c94").unwrap(),
            lighter: Color32::from_hex("#ccd4db").unwrap(),
            success: Color32::from_hex("#22BB33").unwrap(),
            warning: Color32::from_hex("#f0ad4e").unwrap(),
        }
    }

    pub fn light() -> Self {
        Self {
            main: Color32::from_hex("#b0a8a2").unwrap(),
            secondary: Color32::from_hex("#75716d").unwrap(),
            danger: Color32::from_hex("#F7A072").unwrap(),
            light: Color32::from_hex("#d1cbc5").unwrap(),
            lighter: Color32::from_hex("#ede8e4").unwrap(),
            success: Color32::from_hex("#06d6a0").unwrap(),
            warning: Color32::from_hex("#EDDEA4").unwrap(),
        }
    }
}
