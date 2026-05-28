use egui;

pub struct Theme {
    pub heading_sizes: [f32; 6],
    pub code_bg: egui::Color32,
    pub quote_bar_color: egui::Color32,
    pub text_color: egui::Color32,
    pub muted_color: egui::Color32,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            heading_sizes: [28.0, 24.0, 20.0, 18.0, 16.0, 14.0],
            code_bg: egui::Color32::from_gray(40),
            quote_bar_color: egui::Color32::from_gray(100),
            text_color: egui::Color32::from_gray(220),
            muted_color: egui::Color32::from_gray(140),
        }
    }
}
