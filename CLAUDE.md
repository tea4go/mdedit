# CLAUDE.md - mdedit 项目开发指南

## 项目概述

mdedit 是一个基于 Rust + egui/eframe 的轻量级跨平台 Markdown 编辑器，对标 Typora 的原生体验。项目目标是提供零依赖、低延迟、高响应的本地 Markdown 创作体验。

## 技术栈

- **语言**: Rust
- **GUI 框架**: egui / eframe（即时模式 GUI）
- **Markdown 解析**: pulldown-cmark（仅 renderer 模块使用）
- **正则表达式**: regex（搜索功能）
- **文件对话框**: rfd
- **JSON 解析**: serde_json
- **构建系统**: Cargo

## 架构设计

项目采用 MVC 设计模式：
- **Model**: `document/` 模块（Buffer + History + Document）
- **View**: `renderer/`、`editor/`、`toolbar.rs`、`file_tree.rs`、`search.rs`
- **Controller**: `app.rs`（MdEditApp 主结构体，整合所有逻辑）

界面为三栏布局：Ribbon 窄侧栏 | 左面板（目录/大纲/搜索） | 编辑区 | 状态栏

## 编码规范

- 所有代码注释、函数文档、错误输出均使用**中文**
- 每个公开函数必须添加 `///` 文档注释，说明功能和参数含义
- 每个模块文件头部添加 `//!` 模块文档注释
- 重要逻辑和算法处添加行内中文注释
- 使用 `// TODO:` 标记待实现功能

## 构建与运行

```bash
# Windows (MSYS2/MinGW64)
export PATH="/c/msys64/mingw64/bin:$PATH"
export LIBRARY_PATH="/c/msys64/mingw64/lib"
cargo build --release

# 运行
cargo run
cargo run -- 文件.md          # 直接打开文件
cargo run -- --reset          # 重置窗口位置
cargo run -- --debug-theme    # 调试主题
```

## 主题系统

- 主题配置来源：WhaleTerm 的 `preferences.json` 文件
- 回退方案：`CSS_THEME_DIR` 目录下的 `light.css` / `dark.css`
- 最终回退：`theme.rs` 中的硬编码默认值
- 支持三种模式：浅色、深色、跟随系统（通过 Windows 注册表检测）

## 文件操作约定

- 配置文件路径：`%APPDATA%/mdedit/config.ini`（INI 格式）
- 启动日志：`%APPDATA%/mdedit/startup.log`
- 自动保存间隔：60 秒
- 文件树只显示 `.md` 和 `.markdown` 文件

## 编辑模式

- **SV（原始编辑）**: 纯文本 TextEdit，直接编辑 Markdown 源码
- **IR（即时渲染）**: 将文档按语义块分割，非活跃块渲染为富文本，点击进入文本编辑

## 关键注意事项

1. **字节偏移**: Buffer 中的 offset 是字节偏移，非字符偏移，处理中文字符时需注意
2. **DPI 感知**: Windows 上必须设置 DPI Awareness，否则显示模糊
3. **窗口位置**: 保存/恢复使用物理像素坐标，避免多显示器偏移问题
4. **字体加载**: 优先加载 WhaleTerm 配置的字体，回退到系统 CJK 字体
