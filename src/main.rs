#![allow(dead_code)]

mod app;
mod document;
mod editor;
mod outline;
mod renderer;
mod theme;

use eframe::egui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([600.0, 400.0]),
        ..Default::default()
    };

    eframe::run_native(
        "mdedit",
        options,
        Box::new(|cc| Ok(Box::new(app::MdEditApp::new(cc)))),
    )
}
