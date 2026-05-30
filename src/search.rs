use std::path::PathBuf;
use eframe::egui;

// === 编辑器内搜索栏 (SearchBar) ===

pub struct SearchBarState {
    pub visible: bool,
    pub query: String,
    pub replace_text: String,
    pub case_sensitive: bool,
    pub whole_word: bool,
    pub show_replace: bool,
    pub current_match: usize,
    pub total_matches: usize,
}

impl SearchBarState {
    pub fn new() -> Self {
        Self {
            visible: false,
            query: String::new(),
            replace_text: String::new(),
            case_sensitive: false,
            whole_word: false,
            show_replace: false,
            current_match: 0,
            total_matches: 0,
        }
    }

    pub fn count_matches(&mut self, content: &str) {
        if self.query.is_empty() {
            self.total_matches = 0;
            self.current_match = 0;
            return;
        }
        let matches = find_matches(content, &self.query, self.case_sensitive, self.whole_word);
        self.total_matches = matches.len();
        if self.current_match >= self.total_matches {
            self.current_match = 0;
        }
    }

    pub fn matches(&self, content: &str) -> Vec<(usize, usize)> {
        if self.query.is_empty() { return Vec::new(); }
        find_matches(content, &self.query, self.case_sensitive, self.whole_word)
    }
}

fn find_matches(content: &str, query: &str, case_sensitive: bool, whole_word: bool) -> Vec<(usize, usize)> {
    let pattern = if whole_word {
        format!(r"\b{}\b", regex::escape(query))
    } else {
        regex::escape(query).to_string()
    };
    let re = if case_sensitive {
        regex::Regex::new(&pattern).ok()
    } else {
        regex::RegexBuilder::new(&pattern).case_insensitive(true).build().ok()
    };
    let Some(re) = re else { return Vec::new() };
    re.find_iter(content).map(|m| (m.start(), m.end())).collect()
}

pub fn render_search_bar(
    ctx: &egui::Context,
    state: &mut SearchBarState,
    bg_color: egui::Color32,
    border_color: egui::Color32,
    text_color: egui::Color32,
    font_size: f32,
) -> SearchBarAction {
    let mut action = SearchBarAction::None;
    if !state.visible { return action; }

    let bar_width = 37.0_f32.min(ctx.screen_rect().width() * 0.37).max(222.0).min(450.0);

    egui::Area::new(egui::Id::new("search_bar"))
        .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-4.0, 32.0))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            let frame = egui::Frame::default()
                .fill(bg_color)
                .stroke(egui::Stroke::new(1.0, border_color))
                .rounding(egui::Rounding::same(5.0))
                .inner_margin(egui::Margin::symmetric(5.0, 3.0));
            frame.show(ui, |ui| {
                ui.set_min_width(bar_width);
                ui.horizontal(|ui| {
                    // 搜索输入框
                    let textedit = egui::TextEdit::singleline(&mut state.query)
                        .font(egui::FontId::proportional(font_size))
                        .desired_width(bar_width * 0.4)
                        .hint_text("搜索");
                    let resp = ui.add(textedit);
                    if resp.changed() {
                        action = SearchBarAction::QueryChanged;
                    }

                    // 大小写
                    let cs_color = if state.case_sensitive { text_color } else { egui::Color32::GRAY };
                    if ui.add(egui::Button::new(egui::RichText::new("Aa").size(font_size * 0.75).color(cs_color)).frame(state.case_sensitive)).clicked() {
                        state.case_sensitive = !state.case_sensitive;
                    }

                    // 全词
                    let ww_color = if state.whole_word { text_color } else { egui::Color32::GRAY };
                    if ui.add(egui::Button::new(egui::RichText::new("ab").size(font_size * 0.75).color(ww_color)).frame(state.whole_word)).clicked() {
                        state.whole_word = !state.whole_word;
                    }

                    // 匹配计数
                    let count_text = if state.total_matches > 0 {
                        format!("{}/{}", state.current_match + 1, state.total_matches)
                    } else {
                        "0/0".to_string()
                    };
                    ui.label(egui::RichText::new(&count_text).size(font_size * 0.85).color(text_color));

                    // 上/下/关闭
                    if ui.add(egui::Button::new("\u{25B2}").frame(false)).clicked() {
                        action = SearchBarAction::PrevMatch;
                    }
                    if ui.add(egui::Button::new("\u{25BC}").frame(false)).clicked() {
                        action = SearchBarAction::NextMatch;
                    }
                    if ui.add(egui::Button::new("\u{2715}").frame(false)).clicked() {
                        state.visible = false;
                    }
                });

                // 替换行
                if state.show_replace {
                    ui.horizontal(|ui| {
                        let rep = egui::TextEdit::singleline(&mut state.replace_text)
                            .font(egui::FontId::proportional(font_size))
                            .desired_width(bar_width * 0.4)
                            .hint_text("替换");
                        ui.add(rep);
                        if ui.button("替换").clicked() {
                            action = SearchBarAction::Replace;
                        }
                        if ui.button("全部替换").clicked() {
                            action = SearchBarAction::ReplaceAll;
                        }
                    });
                }

                // 切换替换行
                ui.horizontal(|ui| {
                    let toggle_text = if state.show_replace { "隐藏替换" } else { "替换..." };
                    if ui.add(egui::Button::new(toggle_text).frame(false)).clicked() {
                        state.show_replace = !state.show_replace;
                    }
                });
            });
        });

    action
}

#[derive(Clone, PartialEq)]
pub enum SearchBarAction {
    None,
    QueryChanged,
    PrevMatch,
    NextMatch,
    Replace,
    ReplaceAll,
}

// === 全文搜索 (SearchTree) ===

pub struct FileSearchResult {
    pub file_path: PathBuf,
    pub file_name: String,
    pub lines: Vec<LineMatch>,
}

pub struct LineMatch {
    pub line_number: usize,
    pub line_text: String,
    pub match_start: usize,
    pub match_end: usize,
}

pub struct SearchTreeState {
    pub query: String,
    pub results: Vec<FileSearchResult>,
    pub searching: bool,
}

impl SearchTreeState {
    pub fn new() -> Self {
        Self { query: String::new(), results: Vec::new(), searching: false }
    }

    pub fn search(&mut self, query: &str, dir: &std::path::Path) {
        self.query = query.to_string();
        self.results.clear();
        if query.is_empty() { return; }
        self.searching = true;
        search_dir(dir, query, &mut self.results);
        self.searching = false;
    }
}

fn search_dir(dir: &std::path::Path, query: &str, results: &mut Vec<FileSearchResult>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') { continue; }
            if name == "markdown-theme" { continue; }
            if path.is_dir() {
                search_dir(&path, query, results);
            } else if name.ends_with(".md") || name.ends_with(".markdown") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    let matches = find_line_matches(&content, query);
                    if !matches.is_empty() {
                        results.push(FileSearchResult {
                            file_path: path,
                            file_name: name,
                            lines: matches,
                        });
                    }
                }
            }
        }
    }
}

fn find_line_matches(content: &str, query: &str) -> Vec<LineMatch> {
    let pattern = regex::escape(query);
    let re = regex::RegexBuilder::new(&pattern)
        .case_insensitive(true)
        .build();
    let Ok(re) = re else { return Vec::new() };
    let mut matches = Vec::new();
    for (i, line) in content.lines().enumerate() {
        if let Some(m) = re.find(line) {
            matches.push(LineMatch {
                line_number: i,
                line_text: line.to_string(),
                match_start: m.start(),
                match_end: m.end(),
            });
        }
    }
    matches
}

pub struct SearchTreeResult {
    pub open_file: Option<(PathBuf, usize)>,
}

pub fn render_search_tree(
    ui: &mut egui::Ui,
    state: &mut SearchTreeState,
    data_dir: Option<&std::path::Path>,
    text_color: egui::Color32,
    font_size: f32,
    hover_bg: egui::Color32,
) -> SearchTreeResult {
    let mut result = SearchTreeResult { open_file: None };

    // 搜索输入框
    ui.horizontal(|ui| {
        let resp = ui.add(
            egui::TextEdit::singleline(&mut state.query)
                .font(egui::FontId::proportional(font_size))
                .desired_width(ui.available_width() - 60.0)
                .hint_text("搜索内容..."),
        );
        if ui.button("\u{1F50D}").clicked() || resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            if let Some(dir) = data_dir {
                state.search(&state.query.clone(), dir);
            }
        }
    });

    ui.separator();

    // 搜索结果
    if state.searching {
        ui.label(egui::RichText::new("搜索中...").size(font_size).color(text_color));
    } else if state.results.is_empty() {
        if !state.query.is_empty() {
            ui.label(egui::RichText::new("无结果").size(font_size).color(egui::Color32::GRAY));
        }
    } else {
        egui::ScrollArea::vertical()
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
            .show(ui, |ui| {
                for file_result in &state.results {
                    // 文件节点
                    let _file_resp = ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("\u{1F4C4}").size(font_size));
                        ui.label(egui::RichText::new(&file_result.file_name).size(font_size + 2.0).strong().color(text_color));
                    }).response;
                    let file_path = file_result.file_path.clone();

                    // 匹配行
                    for line_match in file_result.lines.iter().take(10) {
                        let line_path = file_path.clone();
                        let line_num = line_match.line_number;
                        let resp = ui.horizontal(|ui| {
                            ui.add_space(20.0);
                            let text = if line_match.line_text.len() > 60 {
                                format!("{}: {}...", line_num + 1, &line_match.line_text[..60])
                            } else {
                                format!("{}: {}", line_num + 1, line_match.line_text)
                            };
                            ui.add(egui::Button::new(
                                egui::RichText::new(text).size(font_size - 1.0).color(text_color)
                            ).frame(false).fill(egui::Color32::TRANSPARENT))
                        }).response;
                        if resp.clicked() {
                            result.open_file = Some((line_path, line_num));
                        }
                        if resp.hovered() {
                            let rect = resp.rect;
                            ui.painter().rect_filled(rect, 0.0, hover_bg);
                        }
                    }
                    ui.separator();
                }
            });
    }

    result
}
