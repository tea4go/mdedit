//! 配置管理模块 - 处理应用配置的加载与保存
//!
//! 配置文件为 INI 格式，存储在 %APPDATA%/mdedit/config.ini 中。
//! 包含窗口位置、大小、主题、编辑模式等设置。

use std::collections::HashMap;
use std::path::PathBuf;

/// 获取配置文件路径
fn config_path() -> PathBuf {
    let mut path = config_dir();
    path.push("config.ini");
    path
}

/// 获取配置文件目录（%APPDATA%/mdedit）
/// 如果 APPDATA 环境变量不存在，回退到当前目录
pub fn config_dir() -> PathBuf {
    if let Some(appdata) = std::env::var_os("APPDATA") {
        let mut p = PathBuf::from(appdata);
        p.push("mdedit");
        p
    } else {
        PathBuf::from(".")
    }
}

/// 应用配置结构体
#[derive(Default)]
pub struct AppConfig {
    /// 窗口 X 坐标（物理像素）
    pub window_x: Option<f32>,
    /// 窗口 Y 坐标（物理像素）
    pub window_y: Option<f32>,
    /// 窗口宽度
    pub window_width: Option<f32>,
    /// 窗口高度
    pub window_height: Option<f32>,
    /// 是否最大化
    pub maximized: bool,
    /// 主题名称（light/dark/auto）
    pub theme: String,
    /// 编辑模式（raw/preview）
    pub edit_mode: String,
}

impl AppConfig {
    /// 从配置文件加载配置，文件不存在时返回默认值
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
            edit_mode: map.get("edit_mode").cloned().unwrap_or_default(),
        }
    }

    /// 将当前配置保存到配置文件
    pub fn save(&self) {
        let dir = config_dir();
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
        if !self.edit_mode.is_empty() {
            lines.push(format!("edit_mode={}", self.edit_mode));
        }
        let content = lines.join("\n");
        let _ = std::fs::write(&path, content);
    }
}

/// 解析 INI 格式文本为键值对 HashMap
/// 支持以 # 开头的注释行和空行
fn parse_ini(content: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for line in content.lines() {
        let line = line.trim();
        // 跳过空行和注释
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            map.insert(k.trim().to_string(), v.trim().to_string());
        }
    }
    map
}