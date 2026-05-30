# mdedit 主题改造计划

> 基于 whaleterm_主题.md 规范，对齐 WhaleTerminal 主题能力（不含主题编辑功能）

---

## 一、P0：笔记主题数据源切换

### 现状
MD 渲染颜色从 CSS 文件加载（`markdown-theme/light.css` / `dark.css`），不读取 preferences.json 中的 `noteThemeLight` / `noteThemeDark`。

### 目标
优先从 `preferences.json` 的 `noteThemeLight`/`noteThemeDark` 读取笔记主题，CSS 文件降为回退方案。

### 实现步骤

#### 1.1 Theme 结构体扩展（src/theme.rs）

新增字段：

```rust
pub struct CodeStyle {
    // ... 现有字段
    pub block_border_color: Color32,   // 代码块边框色
}

pub struct TableStyle {
    // ... 现有字段
    pub border_radius: f32,            // 表格圆角
}
```

#### 1.2 新增 load_note_theme 函数（src/app.rs 或新建 src/theme_loader.rs）

从 preferences.json 的 `noteThemeLight` / `noteThemeDark` 读取并构建 Theme：

```
JSON 字段              → Theme 字段映射：
noteCodeBackgroundColor → code.block_bg
noteCodeBorderColor     → code.block_border_color (新增)
noteCodeBorderRadius    → code.block_rounding
noteMarkerBackgroundColor → code.inline_bg
noteMarkerTextColor     → code.inline_text
noteLinkColor           → link.color
noteQuoteBorderColor    → quote.bar_color
noteQuoteBackgroundColor → quote.bg_color
noteQuoteTextColor      → quote.text_color
noteQuoteBorderWidth    → quote.bar_width
noteTableBgColor        → table.row_bg
noteTableBorderColor    → table.border_color
noteTableHeaderBgColor  → table.header_bg
noteTableEvenRowBgColor → table.alt_row_bg
noteTableBorderRadius   → table.border_radius (新增)
noteH1Color             → heading.colors[0]
noteH2Color             → heading.colors[1]
noteH3Color             → heading.colors[2]
noteH4Color             → heading.colors[3..5]
```

加载优先级：
1. 优先从 `noteThemeLight`/`noteThemeDark` JSON 读取
2. JSON 不可用时回退到 CSS 文件
3. CSS 也不可用时使用 Theme::light() / Theme::dark() 硬编码默认值

#### 1.3 preferences.json 中 noteTheme 实际键名（需确认）

在 `C:\Users\tony\AppData\Roaming\WhaleTerm\preferences.json` 中搜索 `noteThemeLight` 和 `noteThemeDark`，确认实际 JSON 键名格式（可能是驼峰式如 `noteCodeBackgroundColor` 或其他格式）。

#### 1.4 修改构造函数和 switch_theme

```rust
// 构造函数中：
let mut theme = Self::load_note_theme(theme_mode)
    .or_else(|| Self::load_css_theme(theme_mode))
    .unwrap_or_else(|| match theme_mode {
        ThemeMode::Light => Theme::light(),
        ThemeMode::Dark => Theme::dark(),
    });

// switch_theme 中同理
```

#### 1.5 涉及文件
- `src/theme.rs` — 扩展 CodeStyle 和 TableStyle
- `src/app.rs` — 新增 load_note_theme 函数，修改构造函数和 switch_theme
- `src/editor/mod.rs` — 渲染代码块时使用 block_border_color
- `src/renderer/blocks.rs` — 同上

---

## 二、P1：系统主题 UiTheme 字段补充

### 现状
UiTheme 只有 10 个字段，缺少下拉菜单、输入框、激活色等。

### 目标
补充高价值字段，使 UI 颜色与 WhaleTerm 一致。

### 新增字段（src/theme.rs UiTheme）

```rust
pub struct UiTheme {
    // === 现有 10 个字段 ===
    pub menu_bg: Color32,
    pub menu_text: Color32,
    pub sidebar_bg: Color32,
    pub sidebar_text: Color32,
    pub sidebar_hover_bg: Color32,
    pub sidebar_active_bg: Color32,
    pub sidebar_active_text: Color32,
    pub content_bg: Color32,
    pub border: Color32,
    pub divider: Color32,

    // === 新增字段 ===
    pub text_color: Color32,              // textColor — 全局默认前景色（Raw 编辑器）
    pub text_active_color: Color32,       // textActiveColor — 主色调/激活色
    pub split_color: Color32,             // appSplitColor — 大模块分割线
    pub sidebar_active_text_color: Color32, // appSideTextActiveColor
    pub sidebar_hover_text: Color32,      // appLeftListBgColorHover 悬停背景
    pub input_bg: Color32,                // inputContentBgColor — 输入框背景
    pub input_border: Color32,            // inputContentBorderColor — 输入框边框
    pub drop_down_text: Color32,          // dropDownColor — 下拉菜单文本
    pub drop_down_bg: Color32,            // dropDownBgColor — 下拉菜单背景
    pub drop_down_active_text: Color32,   // dropDownActiveColor
    pub drop_down_active_bg: Color32,     // dropDownActiveBgColor
}
```

### 加载映射（load_ui_theme 函数中新增）

```
textColor                → text_color
textActiveColor          → text_active_color
appSplitColor            → split_color
appSideTextActiveColor   → sidebar_active_text_color
appLeftListBgColorHover  → sidebar_hover_text（悬停背景）
inputContentBgColor      → input_bg
inputContentBorderColor  → input_border
dropDownColor            → drop_down_text
dropDownBgColor          → drop_down_bg
dropDownActiveColor      → drop_down_active_text
dropDownActiveBgColor    → drop_down_active_bg
```

### 应用位置
- `text_color` → Raw 编辑器文字颜色（替代当前 sidebar_text）
- `text_active_color` → 菜单项激活/选中色
- `drop_down_*` → 菜单弹出项样式
- `input_bg` / `input_border` → 编辑模式输入框样式

### 涉及文件
- `src/theme.rs` — UiTheme 新增字段
- `src/app.rs` — load_ui_theme 新增映射，渲染时使用新字段

---

## 三、P1：扩展主题色 ExtraTheme

### 现状
未实现任何扩展色。扩展色不存储在 preferences.json，按 light/dark 模式硬编码。

### 目标
新增 mdedit 需要的扩展色结构体。

### 新增结构体（src/theme.rs）

```rust
pub struct ExtraTheme {
    pub outline_hover_color: Color32,     // 大纲面板悬停高亮
    pub note_toolbar_header_bg: Color32,  // 工具栏背景
    pub active_color: Color32,            // 主色调
}
```

### 硬编码默认值

```
亮色：
  outline_hover_color      = "#4285F4"
  note_toolbar_header_bg   = "#F5F5F5"
  active_color             = "#007ACC"

暗色：
  outline_hover_color      = "#FFFFFF"
  note_toolbar_header_bg   = "#023848"
  active_color             = "#FFFFFF"
```

### 涉及文件
- `src/theme.rs` — ExtraTheme 结构体和默认值
- `src/app.rs` — MdEditApp 新增 extra_theme 字段，大纲面板使用 outline_hover_color

---

## 四、P2：RGBA 颜色解析支持

### 现状
`parse_hex_color()` 只支持 `#RGB`（3位）和 `#RRGGBB`（6位）。

### 目标
支持 `#RRGGBBAA`（8位）格式，用于带透明度的颜色（如 `#6680990D`）。

### 修改位置
- `src/app.rs` 的 `parse_hex_color()` 函数（或提取到公共模块）

```rust
fn parse_hex_color(val: &str) -> Option<egui::Color32> {
    let hex = val.trim().strip_prefix('#')?;
    match hex.len() {
        3 => { /* 现有 */ },
        6 => { /* 现有 */ },
        8 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
            Some(egui::Color32::from_rgba_unmultiplied(r, g, b, a))
        }
        _ => None,
    }
}
```

---

## 五、P2：默认值对齐规范

### 现状
UiTheme::default_for() 的硬编码值与规范中的默认主题不一致。

### 目标
亮色默认值使用 "Default Light Modern" 色值，暗色使用 "Solarized Dark" 色值。

### 参考色值（来自 whaleterm_主题.md）

亮色 "Default Light Modern"：
```
text_color = "#333333", text_active_color = "#007ACC"
app_bg_color = "#F5F5F5", app_divider_color = "#E0E0E0"
border_color = "#CCCCCC", app_header_text_color = "#333333"
app_sider_bar_bg_color = "#FFFFFF", app_side_hover_bg_color = "#E3F2FD"
app_side_text_color = "#666666", app_side_text_active_color = "#007ACC"
app_content_note_bg_color = "#FFFFFF"
```

暗色 "Solarized Dark"：
```
text_color = "#FFFFFF", text_active_color = "#FFFFFF"
app_bg_color = "#002B36", app_divider_color = "#073642"
border_color = "#1A7778", app_header_text_color = "#FFFFFF"
app_sider_bar_bg_color = "#073642", app_side_hover_bg_color = "#073642"
app_side_text_color = "#CCCCCC", app_side_text_active_color = "#FFFFFF"
app_content_note_bg_color = "#002B36"
```

### 涉及文件
- `src/app.rs` — UiTheme::default_for() 更新所有默认值

---

## 六、P2：主题模式 auto 跟随系统

### 现状
只支持 light/dark 手动切换。

### 目标
支持 auto 模式，跟随 Windows 系统暗色模式设置。

### 实现方式
- 视图菜单增加"跟随系统"选项
- 读取 `general.theme` 配置（值可能为 "auto"）
- auto 模式下使用 Windows API 检测系统暗色模式
- 系统模式变化时自动切换

### 涉及文件
- `src/app.rs` — ThemeMode 枚举新增 Auto，update() 中检测系统模式
- `src/config.rs` — 存储 auto 配置

---

## 实施顺序

```
Phase 1 (P0): 笔记主题数据源切换
  ├── 1.1 Theme 结构体扩展
  ├── 1.2 确认 preferences.json noteTheme 键名
  ├── 1.3 实现 load_note_theme 函数
  ├── 1.4 修改构造函数和 switch_theme
  └── 1.5 渲染代码块使用新边框色

Phase 2 (P1): 系统主题字段补充
  ├── 2.1 UiTheme 新增 10 个字段
  ├── 2.2 load_ui_theme 新增映射
  └── 2.3 渲染代码使用新字段

Phase 3 (P1): 扩展主题色
  ├── 3.1 ExtraTheme 结构体
  └── 3.2 大纲面板使用 outline_hover_color

Phase 4 (P2): RGBA 解析 + 默认值对齐
  ├── 4.1 parse_hex_color 支持 8 位 hex
  └── 4.2 UiTheme 默认值更新

Phase 5 (P2): auto 模式
  ├── 5.1 ThemeMode 新增 Auto
  └── 5.2 Windows API 检测系统暗色模式
```

---

## 验证方式

每个 Phase 完成后：
1. `cargo build` 编译通过
2. 运行 `cargo run -- "C:\MyWork\AiCode\qterm\whaleterm_主题.md"` 加载测试文档
3. 截图对比 WhaleTerm（`D:\DevDisk\Other\WhaleTerm\QShell.exe`）的视觉效果
4. 切换亮/暗模式验证颜色正确性
5. 切换编辑模式（预览/原始）验证字体和颜色

---

## 关键参考文件

| 文件 | 说明 |
|------|------|
| `C:\Users\tony\AppData\Roaming\WhaleTerm\preferences.json` | 主题配置数据源 |
| `C:\Users\tony\AppData\Roaming\WhaleTerm\mynotes\files\markdown-theme\light.css` | CSS 主题（回退） |
| `C:\Users\tony\AppData\Roaming\WhaleTerm\mynotes\files\markdown-theme\dark.css` | CSS 主题（回退） |
| `D:\DevDisk\Other\WhaleTerm\QShell.exe` | WhaleTerm 参考程序 |
| `D:\MyWork\AiCode\mdedit\whaleterm_主题.md` | 主题规范文档 |
