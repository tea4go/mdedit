//! 文件树组件模块 - 提供文件/目录浏览、右键菜单操作
//!
//! 核心组件：
//! - `FileNode`: 文件树节点（文件或目录）
//! - `FileTreeState`: 文件树状态管理
//! - `render_file_tree`: 渲染文件树 UI

use std::path::{Path, PathBuf};
use eframe::egui;

/// 文件树节点 - 表示一个文件或目录
#[derive(Clone)]
pub struct FileNode {
    /// 文件/目录路径
    pub path: PathBuf,
    /// 显示名称
    pub name: String,
    /// 是否为目录
    pub is_dir: bool,
    /// 子节点列表（仅目录有效）
    pub children: Vec<FileNode>,
    /// 是否已展开（仅目录有效）
    pub expanded: bool,
}

impl FileNode {
    /// 从目录路径创建文件树节点，递归加载子项
    pub fn from_dir(dir: &Path) -> Self {
        let name = dir.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let mut node = FileNode {
            path: dir.to_path_buf(),
            name,
            is_dir: true,
            children: Vec::new(),
            expanded: false,
        };
        node.load_children();
        node
    }

    /// 加载子节点 - 读取目录内容并按自然排序分组（目录在前，文件在后）
    fn load_children(&mut self) {
        if !self.is_dir { return; }
        let mut entries: Vec<std::fs::DirEntry> = Vec::new();
        if let Ok(rd) = std::fs::read_dir(&self.path) {
            for entry in rd.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with('.') { continue; }
                if name == "markdown-theme" { continue; }
                #[cfg(windows)]
                {
                    use std::os::windows::fs::MetadataExt;
                    if let Ok(meta) = entry.metadata() {
                        if meta.file_attributes() & 0x2 != 0 { continue; }
                    }
                }
                entries.push(entry);
            }
        }
        entries.sort_by(|a, b| {
            let a_name = a.file_name().to_string_lossy().to_string();
            let b_name = b.file_name().to_string_lossy().to_string();
            natord_compare(&a_name, &b_name)
        });
        let (dirs, files): (Vec<_>, Vec<_>) = entries.into_iter()
            .partition(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false));

        self.children.clear();
        for entry in dirs.into_iter().chain(files) {
            let name = entry.file_name().to_string_lossy().to_string();
            let path = entry.path();
            let is_dir = path.is_dir();
            let is_md = name.ends_with(".md") || name.ends_with(".markdown");
            if is_dir {
                let mut child = FileNode {
                    path, name, is_dir: true, children: Vec::new(), expanded: false,
                };
                child.load_children();
                self.children.push(child);
            } else if is_md {
                self.children.push(FileNode {
                    path, name, is_dir: false, children: Vec::new(), expanded: false,
                });
            }
        }
    }
}

/// 自然排序比较函数 - 数字按数值比较，非数字按字母比较
fn natord_compare(a: &str, b: &str) -> std::cmp::Ordering {
    let mut ai = a.chars().peekable();
    let mut bi = b.chars().peekable();
    loop {
        let ac = ai.next();
        let bc = bi.next();
        match (ac, bc) {
            (None, None) => return std::cmp::Ordering::Equal,
            (None, _) => return std::cmp::Ordering::Less,
            (_, None) => return std::cmp::Ordering::Greater,
            (Some(x), Some(y)) => {
                if x.is_ascii_digit() && y.is_ascii_digit() {
                    let anum: String = std::iter::once(x).chain(std::iter::from_fn(|| {
                        if ai.peek().map(|c| c.is_ascii_digit()) == Some(true) { ai.next() } else { None }
                    })).collect();
                    let bnum: String = std::iter::once(y).chain(std::iter::from_fn(|| {
                        if bi.peek().map(|c| c.is_ascii_digit()) == Some(true) { bi.next() } else { None }
                    })).collect();
                    let cmp = match (anum.parse::<u64>(), bnum.parse::<u64>()) {
                        (Ok(a), Ok(b)) => a.cmp(&b),
                        _ => anum.cmp(&bnum),
                    };
                    if cmp != std::cmp::Ordering::Equal { return cmp; }
                } else {
                    let al: String = x.to_lowercase().collect();
                    let bl: String = y.to_lowercase().collect();
                    let cmp = al.cmp(&bl);
                    if cmp != std::cmp::Ordering::Equal { return cmp; }
                }
            }
        }
    }
}

/// 扁平化列表项 - 用于渲染时的行数据
pub struct FlatItem {
    /// 缩进深度
    pub depth: usize,
    /// 文件/目录路径
    pub path: PathBuf,
    /// 显示名称
    pub name: String,
    /// 是否为目录
    pub is_dir: bool,
    /// 是否已展开
    pub expanded: bool,
    /// 是否有子项
    pub has_children: bool,
}

/// 文件树状态 - 管理文件树的展开/折叠、选择、剪贴板等状态
pub struct FileTreeState {
    pub root_nodes: Vec<FileNode>,
    pub selected_path: Option<PathBuf>,
    pub data_dir: Option<PathBuf>,
    flat_list: Vec<FlatItem>,
    pub clipboard_path: Option<PathBuf>,
    pub clipboard_is_cut: bool,
    pub clipboard_is_dir: bool,
}

impl FileTreeState {
    pub fn new() -> Self {
        Self {
            root_nodes: Vec::new(),
            selected_path: None,
            data_dir: None,
            flat_list: Vec::new(),
            clipboard_path: None,
            clipboard_is_cut: false,
            clipboard_is_dir: false,
        }
    }

    pub fn set_data_dir(&mut self, dir: PathBuf) {
        self.data_dir = Some(dir.clone());
        self.root_nodes = vec![FileNode::from_dir(&dir)];
        self.rebuild_flat();
    }

    pub fn refresh(&mut self) {
        if let Some(dir) = self.data_dir.clone() {
            self.root_nodes = vec![FileNode::from_dir(&dir)];
            self.rebuild_flat();
        }
    }

    pub fn toggle_expand(&mut self, index: usize) {
        if index >= self.flat_list.len() { return; }
        let path = self.flat_list[index].path.clone();
        let expanded = self.flat_list[index].expanded;
        set_expanded_recursive(&mut self.root_nodes, &path, !expanded);
        self.rebuild_flat();
    }

    pub fn expand_all(&mut self) {
        set_all_expanded_recursive(&mut self.root_nodes, true);
        self.rebuild_flat();
    }

    pub fn collapse_all(&mut self) {
        set_all_expanded_recursive(&mut self.root_nodes, false);
        self.rebuild_flat();
    }

    fn rebuild_flat(&mut self) {
        self.flat_list.clear();
        let nodes = &self.root_nodes;
        let flat = &mut self.flat_list;
        for node in nodes {
            flatten_node_into(node, 0, flat);
        }
    }

    pub fn flat_items(&self) -> &[FlatItem] {
        &self.flat_list
    }
}

fn set_expanded_recursive(nodes: &mut [FileNode], path: &Path, value: bool) {
    for node in nodes.iter_mut() {
        if node.path == path {
            node.expanded = value;
            return;
        }
        set_expanded_recursive(&mut node.children, path, value);
    }
}

fn set_all_expanded_recursive(nodes: &mut [FileNode], value: bool) {
    for node in nodes.iter_mut() {
        node.expanded = value;
        set_all_expanded_recursive(&mut node.children, value);
    }
}

fn flatten_node_into(node: &FileNode, depth: usize, flat: &mut Vec<FlatItem>) {
    flat.push(FlatItem {
        depth,
        path: node.path.clone(),
        name: node.name.clone(),
        is_dir: node.is_dir,
        expanded: node.expanded,
        has_children: !node.children.is_empty(),
    });
    if node.expanded {
        for child in &node.children {
            flatten_node_into(child, depth + 1, flat);
        }
    }
}

/// 文件树操作类型 - 右键菜单操作的结果
#[derive(Clone, PartialEq)]
pub enum FileTreeAction {
    None,
    NewFile(PathBuf),
    NewFolder(PathBuf),
    Delete(PathBuf),
    Rename(PathBuf),
    Refresh,
    Cut(PathBuf),
    Copy(PathBuf),
    Paste(PathBuf),
    ImportMarkdown(PathBuf),
    SearchContent,
    OpenAiChat,
}

pub struct FileTreeResult {
    pub clicked_file: Option<PathBuf>,
    pub action: FileTreeAction,
}

pub fn render_file_tree(
    ui: &mut egui::Ui,
    state: &mut FileTreeState,
    text_color: egui::Color32,
    hover_bg: egui::Color32,
    active_bg: egui::Color32,
    active_text: egui::Color32,
    font_size: f32,
) -> FileTreeResult {
    let mut result = FileTreeResult { clicked_file: None, action: FileTreeAction::None };
    let row_height = font_size * 2.8;

    egui::ScrollArea::vertical()
        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
        .show(ui, |ui| {
            let items_len = state.flat_items().len();
            for idx in 0..items_len {
                let item = &state.flat_items()[idx];
                let indent = item.depth as f32 * 18.0;
                let is_selected = state.selected_path.as_ref() == Some(&item.path);
                let item_path = item.path.clone();
                let item_is_dir = item.is_dir;
                let item_has_children = item.has_children;
                let item_expanded = item.expanded;
                let item_name = item.name.clone();

                let row_response = ui.allocate_response(
                    egui::vec2(ui.available_width(), row_height),
                    egui::Sense::click(),
                );
                let row_rect = row_response.rect;

                if row_response.hovered() {
                    ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
                }
                if is_selected {
                    ui.painter().rect_filled(row_rect, 0.0, active_bg);
                } else if row_response.hovered() {
                    ui.painter().rect_filled(row_rect, 0.0, hover_bg);
                }

                // 绘制行内容
                let painter = ui.painter();
                let mut text_x = row_rect.min.x + indent + 4.0;
                let text_y = row_rect.center().y;

                // 展开/折叠箭头
                if item_is_dir && item_has_children {
                    let arrow = if item_expanded { "\u{25BC}" } else { "\u{25B6}" };
                    let arrow_font = egui::FontId::proportional(font_size * 0.7);
                    let arrow_galley = painter.layout_no_wrap(arrow.to_string(), arrow_font, text_color);
                    painter.text(egui::pos2(text_x, text_y), egui::Align2::LEFT_CENTER, arrow, egui::FontId::proportional(font_size * 0.7), text_color);
                    text_x += arrow_galley.size().x + 4.0;
                    // 箭头点击区域
                    let arrow_rect = egui::Rect::from_min_size(
                        egui::pos2(row_rect.min.x + indent + 4.0, row_rect.min.y),
                        egui::vec2(arrow_galley.size().x + 4.0, row_rect.height()),
                    );
                    let arrow_resp = ui.interact(arrow_rect, ui.id().with(("arrow", idx)), egui::Sense::click());
                    if arrow_resp.clicked() {
                        state.toggle_expand(idx);
                    }
                } else {
                    text_x += font_size * 0.7 + 10.0;
                }

                // 图标
                let dir_color = egui::Color32::from_rgba_unmultiplied(0xFF, 0xCE, 0x78, 191);
                let file_color = egui::Color32::from_rgba_unmultiplied(0x7E, 0xAD, 0xE2, 200);
                let (icon, icon_color) = if item_is_dir {
                    if item_expanded && item_has_children {
                        ("\u{1F4C2}", dir_color)
                    } else {
                        ("\u{1F4C1}", dir_color)
                    }
                } else {
                    ("\u{1F4C4}", file_color)
                };
                let icon_font = egui::FontId::proportional(font_size);
                let icon_galley = painter.layout_no_wrap(icon.to_string(), icon_font.clone(), icon_color);
                painter.text(egui::pos2(text_x, text_y), egui::Align2::LEFT_CENTER, icon, icon_font, icon_color);
                text_x += icon_galley.size().x + 4.0;

                // 文件名
                let name_color = if is_selected { active_text } else {
                    egui::Color32::from_rgba_unmultiplied(text_color.r(), text_color.g(), text_color.b(), if item_is_dir { 255 } else { 230 })
                };
                let name_size = if item_is_dir { font_size + 2.0 } else { font_size };
                painter.text(
                    egui::pos2(text_x, text_y),
                    egui::Align2::LEFT_CENTER,
                    &item_name,
                    egui::FontId::proportional(name_size),
                    name_color,
                );

                if row_response.clicked() {
                    state.selected_path = Some(item_path.clone());
                    if !item_is_dir {
                        result.clicked_file = Some(item_path.clone());
                    }
                }

                row_response.context_menu(|ui| {
                    if item_is_dir {
                        // === 文件夹右键菜单 - 对齐 WhaleTerm 原型 ===
                        if ui.button("展开").clicked() {
                            state.selected_path = Some(item_path.clone());
                            ui.close_menu();
                        }
                        if ui.button("打开AI聊天").clicked() {
                            result.action = FileTreeAction::OpenAiChat;
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.button("新建文件  Ctrl+N").clicked() {
                            result.action = FileTreeAction::NewFile(item_path.clone());
                            ui.close_menu();
                        }
                        if ui.button("导入Markdown").clicked() {
                            result.action = FileTreeAction::ImportMarkdown(item_path.clone());
                            ui.close_menu();
                        }
                        if ui.button("新建分组").clicked() {
                            result.action = FileTreeAction::NewFolder(item_path.clone());
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.button("剪切  Ctrl+X").clicked() {
                            result.action = FileTreeAction::Cut(item_path.clone());
                            ui.close_menu();
                        }
                        if ui.button("复制  Ctrl+C").clicked() {
                            result.action = FileTreeAction::Copy(item_path.clone());
                            ui.close_menu();
                        }
                        let has_clipboard = state.clipboard_path.is_some();
                        if ui.add_enabled(has_clipboard, egui::Button::new("粘贴  Ctrl+V")).clicked() {
                            result.action = FileTreeAction::Paste(item_path.clone());
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.button("重命名  F2").clicked() {
                            result.action = FileTreeAction::Rename(item_path.clone());
                            ui.close_menu();
                        }
                        if ui.button("删除  Del").clicked() {
                            result.action = FileTreeAction::Delete(item_path.clone());
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.button("搜索内容").clicked() {
                            result.action = FileTreeAction::SearchContent;
                            ui.close_menu();
                        }
                        if ui.button("复制文件路径").clicked() {
                            ui.output_mut(|o| o.copied_text = item_path.to_string_lossy().to_string());
                            ui.close_menu();
                        }
                        if ui.button("打开文件位置").clicked() {
                            let _ = std::process::Command::new("explorer").arg(&item_path).spawn();
                            ui.close_menu();
                        }
                    } else {
                        // === 文件右键菜单 - 对齐 WhaleTerm 原型 13 项 ===
                        if ui.button("打开文件").clicked() {
                            result.clicked_file = Some(item_path.clone());
                            ui.close_menu();
                        }
                        if ui.button("打开AI聊天").clicked() {
                            result.action = FileTreeAction::OpenAiChat;
                            ui.close_menu();
                        }
                        ui.separator();
                        if let Some(parent) = item_path.parent() {
                            if ui.button("新建文件  Ctrl+N").clicked() {
                                result.action = FileTreeAction::NewFile(parent.to_path_buf());
                                ui.close_menu();
                            }
                            if ui.button("导入Markdown").clicked() {
                                result.action = FileTreeAction::ImportMarkdown(parent.to_path_buf());
                                ui.close_menu();
                            }
                            if ui.button("新建分组").clicked() {
                                result.action = FileTreeAction::NewFolder(parent.to_path_buf());
                                ui.close_menu();
                            }
                        }
                        ui.separator();
                        if ui.button("剪切  Ctrl+X").clicked() {
                            result.action = FileTreeAction::Cut(item_path.clone());
                            ui.close_menu();
                        }
                        if ui.button("复制  Ctrl+C").clicked() {
                            result.action = FileTreeAction::Copy(item_path.clone());
                            ui.close_menu();
                        }
                        if let Some(parent) = item_path.parent() {
                            let has_clipboard = state.clipboard_path.is_some();
                            if ui.add_enabled(has_clipboard, egui::Button::new("粘贴  Ctrl+V")).clicked() {
                                result.action = FileTreeAction::Paste(parent.to_path_buf());
                                ui.close_menu();
                            }
                        }
                        ui.separator();
                        if ui.button("重命名文件  F2").clicked() {
                            result.action = FileTreeAction::Rename(item_path.clone());
                            ui.close_menu();
                        }
                        if ui.button("删除文件  Del").clicked() {
                            result.action = FileTreeAction::Delete(item_path.clone());
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.button("搜索内容").clicked() {
                            result.action = FileTreeAction::SearchContent;
                            ui.close_menu();
                        }
                        if ui.button("复制文件路径").clicked() {
                            ui.output_mut(|o| o.copied_text = item_path.to_string_lossy().to_string());
                            ui.close_menu();
                        }
                        if ui.button("打开文件位置").clicked() {
                            if let Some(parent) = item_path.parent() {
                                let _ = std::process::Command::new("explorer").arg(parent).spawn();
                            }
                            ui.close_menu();
                        }
                    }
                });
            }

            // 空白区域右键
            let empty_rect = ui.available_rect_before_wrap();
            let empty_resp = ui.interact(empty_rect, ui.id().with("empty"), egui::Sense::click());
            let data_dir = state.data_dir.clone();
            empty_resp.context_menu(|ui| {
                if ui.button("新建文件  Ctrl+N").clicked() {
                    if let Some(dir) = &data_dir {
                        result.action = FileTreeAction::NewFile(dir.clone());
                    }
                    ui.close_menu();
                }
                if ui.button("导入Markdown").clicked() {
                    if let Some(dir) = &data_dir {
                        result.action = FileTreeAction::ImportMarkdown(dir.clone());
                    }
                    ui.close_menu();
                }
                if ui.button("新建分组").clicked() {
                    if let Some(dir) = &data_dir {
                        result.action = FileTreeAction::NewFolder(dir.clone());
                    }
                    ui.close_menu();
                }
                ui.separator();
                let has_clipboard = state.clipboard_path.is_some();
                if ui.add_enabled(has_clipboard, egui::Button::new("粘贴  Ctrl+V")).clicked() {
                    if let Some(dir) = &data_dir {
                        result.action = FileTreeAction::Paste(dir.clone());
                    }
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("搜索内容").clicked() {
                    result.action = FileTreeAction::SearchContent;
                    ui.close_menu();
                }
                if ui.button("刷新").clicked() {
                    result.action = FileTreeAction::Refresh;
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("展开全部").clicked() {
                    state.expand_all();
                    ui.close_menu();
                }
                if ui.button("折叠全部").clicked() {
                    state.collapse_all();
                    ui.close_menu();
                }
            });
        });

    result
}
