use std::collections::HashMap;
use std::path::PathBuf;

fn config_path() -> PathBuf {
    let mut path = dirs();
    path.push("config.ini");
    path
}

fn dirs() -> PathBuf {
    if let Some(appdata) = std::env::var_os("APPDATA") {
        let mut p = PathBuf::from(appdata);
        p.push("mdedit");
        p
    } else {
        PathBuf::from(".")
    }
}

#[derive(Default)]
pub struct AppConfig {
    pub window_x: Option<f32>,
    pub window_y: Option<f32>,
    pub window_width: Option<f32>,
    pub window_height: Option<f32>,
    pub maximized: bool,
    pub theme: String,
}

impl AppConfig {
    pub fn load() -> Self {
        let path = config_path();
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => return Self::default(),
        };
        let map = parse_ini(&content);
        Self {
            window_x: map.get("window_x").and_then(|v| v.parse().ok()),
            window_y: map.get("window_y").and_then(|v| v.parse().ok()),
            window_width: map.get("window_width").and_then(|v| v.parse().ok()),
            window_height: map.get("window_height").and_then(|v| v.parse().ok()),
            maximized: map.get("maximized").map(|v| v == "true").unwrap_or(false),
            theme: map.get("theme").cloned().unwrap_or_default(),
        }
    }

    pub fn save(&self) {
        let dir = dirs();
        let _ = std::fs::create_dir_all(&dir);
        let path = config_path();
        let mut lines = Vec::new();
        if let Some(x) = self.window_x {
            lines.push(format!("window_x={}", x));
        }
        if let Some(y) = self.window_y {
            lines.push(format!("window_y={}", y));
        }
        if let Some(w) = self.window_width {
            lines.push(format!("window_width={}", w));
        }
        if let Some(h) = self.window_height {
            lines.push(format!("window_height={}", h));
        }
        lines.push(format!("maximized={}", self.maximized));
        if !self.theme.is_empty() {
            lines.push(format!("theme={}", self.theme));
        }
        let content = lines.join("\n");
        let _ = std::fs::write(&path, content);
    }
}

fn parse_ini(content: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            map.insert(k.trim().to_string(), v.trim().to_string());
        }
    }
    map
}