# mdedit

轻量级跨平台 Markdown 编辑器，Typora 式所见即所得，无需 WebView2。

## 特性

- 所见即所得（WYSIWYG）Markdown 编辑
- 大纲面板：实时标题导航
- 跨平台：Windows / macOS / Linux
- 单文件分发，体积 < 4MB
- 冷启动 < 200ms

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
# 或直接运行
./target/release/mdedit
```

## 快捷键

| 快捷键 | 功能 |
|--------|------|
| Ctrl+N | 新建文档 |
| Ctrl+O | 打开文件 |
| Ctrl+S | 保存文件 |

## 许可证

MIT
