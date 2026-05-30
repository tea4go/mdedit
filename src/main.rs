#![allow(dead_code)]

mod app;
mod config;
mod css_loader;
mod document;
mod editor;
mod file_tree;
mod outline;
mod renderer;
mod search;
mod theme;
mod toolbar;

use eframe::egui;
use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

const CSS_THEME_DIR: &str =
    r"C:\Users\tony\AppData\Roaming\WhaleTerm\mynotes\files\markdown-theme";

fn log_startup(msg: &str) {
    let log_path = config::config_dir().join("startup.log");
    if let Ok(mut f) = fs::OpenOptions::new().create(true).append(true).open(&log_path) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let _ = writeln!(f, "[{}] {}", now, msg);
    }
}

#[cfg(windows)]
fn get_dpi_scale() -> f32 {
    extern "system" {
        fn GetDpiForSystem() -> u32;
    }
    let dpi = unsafe { GetDpiForSystem() };
    dpi as f32 / 96.0
}

#[cfg(windows)]
fn log_monitors() {
    use std::sync::Mutex;
    static MONITOR_INFO: Mutex<Vec<String>> = Mutex::new(Vec::new());

    #[repr(C)]
    struct MONITORINFOEXW {
        cb_size: u32,
        rc_monitor: [i32; 4],  // left, top, right, bottom
        rc_work: [i32; 4],
        dw_flags: u32,
        sz_device: [u16; 32],
    }

    extern "system" fn enum_callback(
        hmonitor: usize, _hdc: usize, _lprect: usize, _lparam: isize
    ) -> i32 {
        extern "system" {
            fn GetMonitorInfoW(hMonitor: usize, lpmi: *mut MONITORINFOEXW) -> i32;
            fn GetDpiForMonitor(
                hmonitor: usize, dpi_type: u32,
                dpi_x: *mut u32, dpi_y: *mut u32,
            ) -> i32;
        }
        let mut info: MONITORINFOEXW = unsafe { std::mem::zeroed() };
        info.cb_size = std::mem::size_of::<MONITORINFOEXW>() as u32;
        unsafe { GetMonitorInfoW(hmonitor, &mut info); }

        let mut dpi_x: u32 = 0;
        let mut dpi_y: u32 = 0;
        unsafe { GetDpiForMonitor(hmonitor, 0, &mut dpi_x, &mut dpi_y); }

        let primary = if info.dw_flags & 1 != 0 { " [主屏]" } else { "" };
        let msg = format!(
            "  显示器 {:#x}: 区域=({},{})~({},{}), 工作区=({},{})~({},{}), DPI={}x{}, 缩放={}%{}",
            hmonitor,
            info.rc_monitor[0], info.rc_monitor[1],
            info.rc_monitor[2], info.rc_monitor[3],
            info.rc_work[0], info.rc_work[1],
            info.rc_work[2], info.rc_work[3],
            dpi_x, dpi_y, dpi_x * 100 / 96, primary,
        );
        MONITOR_INFO.lock().unwrap().push(msg);
        1 // continue
    }

    extern "system" {
        fn EnumDisplayMonitors(
            hdc: usize, lprc: usize,
            lpfn: extern "system" fn(usize, usize, usize, isize) -> i32,
            dwdata: isize,
        ) -> i32;
    }

    MONITOR_INFO.lock().unwrap().clear();
    unsafe { EnumDisplayMonitors(0, 0, enum_callback, 0); }

    let infos = MONITOR_INFO.lock().unwrap();
    log_startup(&format!("显示器 (共{}个):", infos.len()));
    for info in infos.iter() {
        log_startup(info);
    }
}

#[cfg(windows)]
fn is_position_visible(x: f32, y: f32, w: f32, h: f32) -> bool {
    #[repr(C)]
    struct RECT { left: i32, top: i32, right: i32, bottom: i32 }
    extern "system" {
        fn MonitorFromRect(lprc: *const RECT, dwFlags: u32) -> usize;
    }
    const MONITOR_DEFAULTTONULL: u32 = 0;
    let rc = RECT {
        left: x as i32,
        top: y as i32,
        right: (x + w) as i32,
        bottom: (y + h) as i32,
    };
    let monitor = unsafe { MonitorFromRect(&rc, MONITOR_DEFAULTTONULL) };
    let visible = monitor != 0;
    log_startup(&format!(
        "MonitorFromRect(left={}, top={}, right={}, bottom={}) => monitor={:#x}, 可见={}",
        rc.left, rc.top, rc.right, rc.bottom, monitor, visible
    ));
    visible
}

#[cfg(not(windows))]
fn is_position_visible(x: f32, y: f32, _w: f32, _h: f32) -> bool {
    x >= 0.0 && y >= 0.0 && x < 5000.0 && y < 3000.0
}

fn parse_setpos() -> Option<(f32, f32)> {
    let args: Vec<String> = env::args().collect();
    let pos = args.iter().position(|a| a == "--setpos")?;
    let val = args.get(pos + 1)?;
    let parts: Vec<&str> = val.split(',').collect();
    if parts.len() == 2 {
        let x = parts[0].trim().parse::<f32>().ok()?;
        let y = parts[1].trim().parse::<f32>().ok()?;
        Some((x, y))
    } else {
        None
    }
}

fn load_initial_file() -> Option<(PathBuf, String)> {
    let args: Vec<String> = env::args().collect();
    // 跳过 --xxx 参数及其值（如 --setpos 3840,222、--debug-theme light）
    let mut skip_next = false;
    let file_arg = args.iter().skip(1).find(|a| {
        if skip_next {
            skip_next = false;
            return false;
        }
        if *a == "--setpos" || *a == "--debug-theme" {
            skip_next = true;
            return false;
        }
        !a.starts_with("--")
    })?;

    let path = PathBuf::from(file_arg);
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
    // 必须在 winit 初始化前声明 DPI Awareness
    #[cfg(windows)]
    {
        extern "system" {
            fn SetProcessDpiAwarenessContext(value: isize) -> i32;
        }
        const DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2: isize = -4;
        unsafe { SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2); }
    }

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
    let reset = env::args().any(|a| a == "--reset");
    let setpos = parse_setpos();
    let cfg = config::AppConfig::load();

    log_startup("========== mdedit 启动 ==========");

    // 打印所有显示器的坐标和 DPI
    #[cfg(windows)]
    log_monitors();

    log_startup(&format!(
        "配置: x={:?}, y={:?}, w={:?}, h={:?}, maximized={}",
        cfg.window_x, cfg.window_y, cfg.window_width, cfg.window_height, cfg.maximized
    ));

    let mut viewport = egui::ViewportBuilder::default()
        .with_min_inner_size([600.0, 400.0]);

    #[cfg(windows)]
    let scale = get_dpi_scale();
    #[cfg(not(windows))]
    let scale = 1.0f32;

    log_startup(&format!("系统 DPI 缩放={}", scale));

    let w = cfg.window_width.unwrap_or(1200.0).max(600.0);
    let h = cfg.window_height.unwrap_or(800.0).max(400.0);

    if reset {
        log_startup("--reset: 重置窗口到主屏居中");
        viewport = viewport.with_inner_size([1200.0, 800.0]);
    } else if let Some((x, y)) = setpos {
        log_startup(&format!("--setpos: 物理坐标({}, {})", x, y));
        viewport = viewport.with_inner_size([w, h]);
    } else {
        viewport = viewport.with_inner_size([w, h]);

        if let (Some(x), Some(y)) = (cfg.window_x, cfg.window_y) {
            if is_position_visible(x, y, w, h) {
                log_startup(&format!(
                    "恢复窗口位置: 物理({}, {}), 大小: ({}, {})",
                    x, y, w, h
                ));
            } else {
                log_startup(&format!(
                    "位置 ({}, {}) 不在可见区域，使用系统默认位置", x, y
                ));
            }
        } else {
            log_startup("无保存的位置，使用系统默认位置");
        }
        if cfg.maximized {
            viewport = viewport.with_maximized(true);
        }
    }

    // 确定目标物理位置
    let target_pos: Option<(f32, f32)> = if reset {
        None
    } else if let Some(pos) = setpos {
        Some(pos)
    } else if let (Some(x), Some(y)) = (cfg.window_x, cfg.window_y) {
        if is_position_visible(x, y, w, h) { Some((x, y)) } else { None }
    } else {
        None
    };

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "mdedit",
        options,
        Box::new(move |cc| Ok(Box::new(app::MdEditApp::new(cc, initial_file, &cfg, target_pos)))),
    )
}
