# mdedit

轻量级跨平台 Markdown 编辑器，Typora 式所见即所得体验，无需 WebView2 依赖。

## 特性

- **所见即所得（WYSIWYG）** Markdown 编辑 - 即时渲染预览模式
- **大纲面板** - 基于标题层级的实时导航，支持展开级别控制和编号显示
- **文件树** - 侧边栏文件目录浏览，支持新建/删除/重命名/剪切/复制/粘贴
- **全文搜索** - 跨文件内容搜索，快速定位目标段落
- **主题系统** - 浅色/深色/跟随系统三种模式，支持 WhaleTerm 主题配置
- **自动保存** - 编辑后 60 秒自动保存，防止数据丢失
- **跨平台** - Windows / macOS / Linux
- **极致轻量** - 单文件 < 4MB，冷启动 < 200ms

## 编辑模式

| 模式 | 说明 |
|------|------|
| SV（原始编辑） | 纯 Markdown 文本编辑，适合精确控制格式 |
| IR（即时渲染） | 点击块级元素进入编辑，其余部分实时渲染预览 |

## 构建

### 依赖

- Rust 1.70+
- Windows (GNU): 需要 MSYS2 MinGW64 工具链

### 编译

```bash
# Windows (MSYS2/MinGW64 环境)
export PATH="/c/msys64/mingw64/bin:$PATH"
export LIBRARY_PATH="/c/msys64/mingw64/lib"
cargo build --release
```

### 运行

```bash
cargo run
# 或直接运行编译产物
./target/release/mdedit
```

### 命令行参数

| 参数 | 说明 |
|------|------|
| `文件路径` | 直接打开指定 Markdown 文件 |
| `--reset` | 重置窗口位置到主屏居中 |
| `--setpos x,y` | 设置窗口物理坐标（供外部调用） |
| `--debug-theme [light/dark]` | 调试：输出 CSS 主题解析结果 |

## 快捷键

| 快捷键 | 功能 |
|--------|------|
| Ctrl+N | 新建文档 |
| Ctrl+O | 打开文件 |
| Ctrl+S | 保存文件 |
| Ctrl+Shift+S | 另存为 |
| Ctrl+E | 切换编辑模式（SV/IR） |
| Ctrl+B | 加粗 |
| Ctrl+I | 斜体 |
| Ctrl+F | 搜索 |
| Ctrl+Plus / Ctrl+Minus | 字体缩放 |

## 项目结构

```
src/
├── main.rs          # 程序入口，窗口初始化和命令行解析
├── app.rs           # 应用主逻辑，状态管理和界面布局
├── auto_save.rs     # 自动保存模块
├── config.rs        # 配置管理（窗口位置、主题、编辑模式）
├── css_loader.rs    # CSS 主题加载和解析
├── document/        # 文档模型
│   ├── mod.rs       #   文档结构体
│   ├── buffer.rs    #   文本缓冲区
│   └── history.rs   #   编辑历史（撤销/重做）
├── editor/          # 编辑器
│   └── mod.rs       #   文本块分割与富文本渲染
├── file_tree.rs     # 文件树组件
├── outline/         # 大纲导航
│   └── mod.rs       #   标题提取和导航状态
├── renderer/        # Markdown 渲染引擎
│   ├── mod.rs       #   pulldown-cmark 解析
│   ├── blocks.rs    #   块级元素渲染
│   └── inline.rs    #   行内格式渲染（预留）
├── search.rs        # 搜索功能（编辑器内搜索 + 全文搜索）
├── theme.rs         # 主题样式定义
└── toolbar.rs       # 工具栏组件
```

## 许可证

MIT
