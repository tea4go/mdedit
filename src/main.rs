#![allow(dead_code)]

mod app;
mod config;
mod css_loader;
mod document;
mod editor;
mod outline;
mod renderer;
mod theme;

use eframe::egui;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const CSS_THEME_DIR: &str =
    r"C:\Users\tony\AppData\Roaming\WhaleTerm\mynotes\files\markdown-theme";

#[cfg(windows)]
fn is_position_visible(x: f32, y: f32, _w: f32, _h: f32) -> bool {
    #[repr(C)]
    struct POINT { x: i32, y: i32 }
    extern "system" {
        fn MonitorFromPoint(pt: POINT, dwFlags: u32) -> usize;
    }
    const MONITOR_DEFAULTTONULL: u32 = 0;
    let pt = POINT { x: x as i32 + 50, y: y as i32 + 50 };
    let monitor = unsafe { MonitorFromPoint(pt, MONITOR_DEFAULTTONULL) };
    monitor != 0
}

#[cfg(not(windows))]
fn is_position_visible(x: f32, y: f32, _w: f32, _h: f32) -> bool {
    x >= 0.0 && y >= 0.0 && x < 5000.0 && y < 3000.0
}

fn load_initial_file() -> Option<(PathBuf, String)> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return None;
    }

    let path = PathBuf::from(&args[1]);
    match fs::read_to_string(&path) {
        Ok(content) => Some((path, content)),
        Err(e) => {
            rfd::MessageDialog::new()
                .set_title("错误")
                .set_description(&format!("无法打开文件：{}\n\n{}", path.display(), e))
                .set_buttons(rfd::MessageButtons::Ok)
                .show();
            None
        }
    }
}

fn main() -> eframe::Result<()> {
    // 调试：输出 CSS 主题解析结果
    if env::args().any(|a| a == "--debug-theme") {
        let name = env::args()
            .skip_while(|a| a != "--debug-theme")
            .nth(1)
            .unwrap_or_else(|| "light".to_string());
        let path = Path::new(CSS_THEME_DIR).join(format!("{}.css", name));
        if let Some(theme) = css_loader::load_theme_from_css(&path) {
            println!("{}", css_loader::debug_theme(&theme));
        } else {
            println!("Failed to load CSS theme from: {}", path.display());
        }
        return Ok(());
    }

    let initial_file = load_initial_file();
    let cfg = config::AppConfig::load();

    let mut viewport = egui::ViewportBuilder::default()
        .with_min_inner_size([600.0, 400.0]);

    let w = cfg.window_width.unwrap_or(1200.0).max(600.0);
    let h = cfg.window_height.unwrap_or(800.0).max(400.0);
    viewport = viewport.with_inner_size([w, h]);

    if let (Some(x), Some(y)) = (cfg.window_x, cfg.window_y) {
        if is_position_visible(x, y, w, h) {
            viewport = viewport.with_position([x, y]);
        }
    }
    if cfg.maximized {
        viewport = viewport.with_maximized(true);
    }

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "mdedit",
        options,
        Box::new(move |cc| Ok(Box::new(app::MdEditApp::new(cc, initial_file, &cfg)))),
    )
}
