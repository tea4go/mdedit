# mdedit 设计文档

## 1. 架构概览

```
┌─────────────────────────────────────────────────────────┐
│                    eframe (egui)                         │
├──────────┬────────────────────────────┬─────────────────┤
│  大纲面板 │       编辑区 (WYSIWYG)      │    工具栏       │
│ Outline  │      Editor Panel          │   Toolbar       │
├──────────┴────────────────────────────┴─────────────────┤
│                    核心层 (Core)                          │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐              │
│  │ Document │  │ Renderer │  │  Parser  │              │
│  │  Model   │  │  Engine  │  │(pulldown │              │
│  │          │  │          │  │  -cmark) │              │
│  └──────────┘  └──────────┘  └──────────┘              │
├─────────────────────────────────────────────────────────┤
│                    基础层 (Foundation)                    │
│  文件 I/O  │  撤销/重做栈  │  配置管理  │  快捷键系统    │
└─────────────────────────────────────────────────────────┘
```

## 2. 模块设计

### 2.1 模块划分

```
src/
├── main.rs              # 入口，初始化 eframe
├── app.rs               # MdEditApp 主结构，实现 eframe::App
├── document/
│   ├── mod.rs           # Document 模型
│   ├── buffer.rs        # 文本缓冲区（rope 或 String）
│   └── history.rs       # 撤销/重做栈
├── editor/
│   ├── mod.rs           # 编辑器主逻辑
│   ├── input.rs         # 键盘/鼠标输入处理
│   └── cursor.rs        # 光标与选区管理
├── renderer/
│   ├── mod.rs           # WYSIWYG 渲染引擎
│   ├── blocks.rs        # 块级元素渲染（标题、代码块、列表等）
│   └── inline.rs        # 行内元素渲染（粗体、斜体、链接等）
├── outline/
│   └── mod.rs           # 大纲面板
├── toolbar.rs           # 工具栏
├── file_ops.rs          # 文件打开/保存
├── config.rs            # 配置管理
└── theme.rs             # 主题（深色/浅色）
```

### 2.2 核心数据结构

```rust
/// 文档模型
pub struct Document {
    pub path: Option<PathBuf>,   // 文件路径，None 表示未保存的新文档
    pub content: String,         // 原始 Markdown 文本
    pub modified: bool,          // 是否有未保存修改
    pub history: History,        // 撤销/重做栈
}

/// 撤销/重做栈
pub struct History {
    undo_stack: Vec<EditOp>,
    redo_stack: Vec<EditOp>,
}

pub struct EditOp {
    pub offset: usize,           // 修改起始位置
    pub old_text: String,        // 被替换的文本
    pub new_text: String,        // 替换后的文本
}

/// 大纲节点
pub struct OutlineItem {
    pub level: u8,               // 1-6
    pub title: String,           // 标题文本
    pub line: usize,             // 源文本行号
}

/// 应用主状态
pub struct MdEditApp {
    pub document: Document,
    pub outline: Vec<OutlineItem>,
    pub cursor_line: usize,      // 当前光标所在行
    pub show_outline: bool,      // 是否显示大纲面板
    pub theme: Theme,
}
```

### 2.3 WYSIWYG 渲染策略

**核心思路**：Typora 模式 = 渲染视图 + 光标所在块回退为源码。

```
状态机：
  [渲染模式] ──光标进入块──▶ [源码模式]
  [源码模式] ──光标离开块──▶ [渲染模式]
```

实现方式：
1. 将文档按块（block）切分：段落、标题、代码块、列表等
2. 每个块独立判断：光标是否在此块内
3. 光标所在块 → 显示原始 Markdown 文本（可编辑）
4. 其他块 → 解析为 AST 后渲染为富文本（只读外观）

### 2.4 数据流

```
用户输入 → 修改 content(String) → 重新解析当前块 → 更新渲染
                                 → 更新大纲（如果标题变化）
                                 → 压入 History 栈
```

## 3. 技术选型细节

| 组件 | 选择 | 版本 | 理由 |
|------|------|------|------|
| GUI 框架 | eframe/egui | 0.29+ | 跨平台、immediate mode、单二进制 |
| Markdown 解析 | pulldown-cmark | 0.12+ | 纯 Rust、CommonMark 兼容、GFM 扩展 |
| 语法高亮 | syntect | 5.x | 支持 200+ 语言、TextMate 语法 |
| 文件对话框 | rfd | 0.15+ | 原生系统对话框、跨平台 |
| 图片解码 | image | 0.25+ | 支持 PNG/JPEG/GIF/WebP |

## 4. 关键设计决策

### 4.1 文本存储：String vs Rope

MVP 阶段使用 `String`：
- 实现简单，5MB 以下文档性能足够
- 后续如需支持超大文档，可替换为 `ropey` crate

### 4.2 渲染粒度：按块渲染

不逐字符渲染，而是按 Markdown 块（block）为单位：
- 减少每帧渲染计算量
- 光标移动时只需重绘 2 个块（离开的块 + 进入的块）
- 块内编辑只影响当前块的重新解析

### 4.3 编辑模型：直接操作源文本

用户的所有编辑操作最终都映射为对 `content: String` 的修改：
- 保证保存时输出的是标准 Markdown
- 不存在"渲染状态"与"源码状态"不一致的问题
- 撤销/重做直接操作文本 diff

## 5. MVP 实现计划

| 阶段 | 交付物 | 验证标准 |
|------|--------|----------|
| 1 | 窗口 + 文本编辑区 | 能输入文本、光标移动正常 |
| 2 | Markdown 渲染 | 标题/粗斜体/列表正确渲染 |
| 3 | 块级切换 | 光标所在块显示源码，其他块渲染 |
| 4 | 大纲面板 | 左侧显示标题树，点击跳转 |
| 5 | 文件操作 | 打开/保存 .md 文件 |
| 6 | 撤销/重做 | Ctrl+Z/Y 正常工作 |
