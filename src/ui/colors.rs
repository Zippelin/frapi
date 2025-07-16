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

impl ThemeColors {
    pub fn dark() -> Self {
        Self {
            main: Color32::from_hex("#212529").unwrap(),
            secondary: Color32::from_hex("#495057").unwrap(),
            danger: Color32::from_hex("#ee6c4d").unwrap(),
            light: Color32::from_hex("#6c757d").unwrap(),
            lighter: Color32::from_hex("#adb5bd").unwrap(),
            success: Color32::from_hex("#22BB33").unwrap(),
            warning: Color32::from_hex("#f0ad4e").unwrap(),
        }
    }

    // TODO: добавить цвета светлой темы
    pub fn light() -> Self {
        Self {
            main: Color32::from_hex("#212529").unwrap(),
            secondary: Color32::from_hex("#495057").unwrap(),
            danger: Color32::from_hex("#ee6c4d").unwrap(),
            light: Color32::from_hex("#6c757d").unwrap(),
            lighter: Color32::from_hex("#adb5bd").unwrap(),
            success: Color32::from_hex("#22BB33").unwrap(),
            warning: Color32::from_hex("#f0ad4e").unwrap(),
        }
    }
}
