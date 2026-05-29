# 命令行打开文件实现计划

> **面向 AI 代理的工作者：** 必需子技能：使用 superpowers:subagent-driven-development（推荐）或 superpowers:executing-plans 逐任务实现此计划。步骤使用复选框（`- [ ]`）语法来跟踪进度。

**目标：** 为 mdedit 添加命令行参数打开文件能力，支持 `mdedit.exe README.md` 直接打开指定文件

**架构：** 在 `main.rs` 入口解析命令行参数并读取文件，通过 `Option<(PathBuf, String)>` 传给 `MdEditApp::new()`，失败时弹错误框并回退到空白页

**技术栈：** Rust、eframe、rfd

---

## 文件结构

### 修改文件

**`src/main.rs`**
- 职责：程序入口，解析命令行参数，读取文件，错误提示，初始化应用
- 新增：`load_initial_file()` 函数，返回 `Option<(PathBuf, String)>`
- 新增：错误提示框逻辑

**`src/app.rs`**
- 职责：应用主逻辑，文档管理，UI 渲染
- 修改：`MdEditApp::new()` 签名，接收 `initial_file: Option<(PathBuf, String)>`
- 修改：根据 `initial_file` 初始化 `document` 和 `outline_items`

### 不修改文件

- `src/document/mod.rs` - 已有 `Document::new()` 和 `Document::from_file()`
- `src/outline/mod.rs` - 已有 `extract_outline()`
- `src/editor/mod.rs` - 渲染逻辑不变
- `src/renderer/mod.rs` - 渲染逻辑不变
- `src/theme.rs` - 主题不变

---

## 任务 1：在 main.rs 添加命令行参数解析

**文件：**
- 修改：`src/main.rs:1-25`

- [ ] **步骤 1：添加必要的 use 声明**

在 `src/main.rs` 文件开头添加：

```rust
use std::env;
use std::fs;
use std::path::PathBuf;
```

- [ ] **步骤 2：编写 load_initial_file 函数**

在 `main()` 函数之前添加：

```rust
fn load_initial_file() -> Option<(PathBuf, String)> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return None;
    }

    let path = PathBuf::from(&args[1]);
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
```

- [ ] **步骤 3：验证编译**

运行：

```bash
cd /c/MyWork/AiCode/mdedit
cargo build
```

预期：编译成功，但 `load_initial_file()` 未被调用会有警告

- [ ] **步骤 4：Commit**

```bash
git add src/main.rs
git commit -m "feat: 添加命令行参数解析函数

新增 load_initial_file() 函数，解析第一个命令行参数作为文件路径，
读取成功返回 Some((PathBuf, String))，失败弹错误框并返回 None。

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## 任务 2：修改 MdEditApp::new() 签名

**文件：**
- 修改：`src/app.rs:18-29`

- [ ] **步骤 1：修改 new() 函数签名**

将 `src/app.rs:18` 的签名从：

```rust
pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
```

改为：

```rust
pub fn new(cc: &eframe::CreationContext<'_>, initial_file: Option<(PathBuf, String)>) -> Self {
```

同时在文件开头添加：

```rust
use std::path::PathBuf;
```

- [ ] **步骤 2：根据 initial_file 初始化 document 和 outline_items**

将 `src/app.rs:20-22` 的初始化逻辑从：

```rust
Self {
    document: Document::new(),
    outline_items: Vec::new(),
```

改为：

```rust
let (document, outline_items) = match initial_file {
    Some((path, content)) => {
        let doc = Document::from_file(path, content);
        let outline = outline::extract_outline(doc.content());
        (doc, outline)
    }
    None => (Document::new(), Vec::new()),
};

Self {
    document,
    outline_items,
```

- [ ] **步骤 3：验证编译**

运行：

```bash
cd /c/MyWork/AiCode/mdedit
cargo build
```

预期：编译失败，报错 `main.rs` 中调用 `MdEditApp::new()` 缺少参数

- [ ] **步骤 4：Commit**

```bash
git add src/app.rs
git commit -m "feat: MdEditApp::new() 支持初始文件参数

修改构造函数签名，接收 initial_file: Option<(PathBuf, String)>，
根据参数初始化 document 和 outline_items。

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## 任务 3：在 main.rs 中调用 load_initial_file 并传给 MdEditApp

**文件：**
- 修改：`src/main.rs:12-24`

- [ ] **步骤 1：在 main() 中调用 load_initial_file**

将 `src/main.rs:20-23` 的 `eframe::run_native` 调用从：

```rust
eframe::run_native(
    "mdedit",
    options,
    Box::new(|cc| Ok(Box::new(app::MdEditApp::new(cc)))),
)
```

改为：

```rust
let initial_file = load_initial_file();
eframe::run_native(
    "mdedit",
    options,
    Box::new(move |cc| Ok(Box::new(app::MdEditApp::new(cc, initial_file)))),
)
```

- [ ] **步骤 2：验证编译**

运行：

```bash
cd /c/MyWork/AiCode/mdedit
cargo build --release
```

预期：编译成功，无警告

- [ ] **步骤 3：Commit**

```bash
git add src/main.rs
git commit -m "feat: 连接命令行参数解析与应用初始化

在 main() 中调用 load_initial_file() 并将结果传给 MdEditApp::new()，
完成命令行打开文件功能。

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## 任务 4：手工测试验证

**文件：**
- 测试：手工验证

- [ ] **步骤 1：测试无参数启动**

运行：

```bash
cd /c/MyWork/AiCode/mdedit
./target/release/mdedit.exe
```

预期：
- 程序正常启动
- 标题栏显示 `未命名 - mdedit`
- 编辑区为空白页
- 大纲面板为空

- [ ] **步骤 2：测试打开存在的文件**

运行：

```bash
cd /c/MyWork/AiCode/mdedit
./target/release/mdedit.exe README.md
```

预期：
- 程序正常启动
- 标题栏显示 `README.md - mdedit`
- 编辑区显示 README.md 内容
- 左侧大纲面板显示标题列表（如 "# mdedit"、"## 特性" 等）
- 点击大纲项可跳转到对应标题

- [ ] **步骤 3：测试打开不存在的文件**

运行：

```bash
cd /c/MyWork/AiCode/mdedit
./target/release/mdedit.exe not-exist.md
```

预期：
- 弹出错误提示框，标题为 "错误"
- 提示内容包含 "无法打开文件：not-exist.md" 和具体错误信息
- 点击 "确定" 后，程序继续启动
- 标题栏显示 `未命名 - mdedit`
- 编辑区为空白页

- [ ] **步骤 4：回归测试现有功能**

在打开 README.md 后，依次测试：

1. **Ctrl+O**：弹出文件对话框，可正常选择文件
2. **Ctrl+S**：保存当前文档
3. **Ctrl+N**：新建空白文档
4. **大纲面板**：点击标题可跳转
5. **WYSIWYG 渲染**：点击块进入编辑模式，失焦后恢复渲染

预期：所有功能正常工作

- [ ] **步骤 5：记录测试结果**

创建测试记录文件：

```bash
cat > /c/MyWork/AiCode/mdedit/docs/superpowers/test-results/2026-05-28-cli-open-file.md <<'EOF'
# 命令行打开文件测试结果

## 测试时间
2026-05-28

## 测试环境
- OS: Windows 11
- Rust: 1.70+
- 构建: release

## 测试用例

### 用例 1：无参数启动
- 命令：`mdedit.exe`
- 结果：✓ 通过
- 标题栏：`未命名 - mdedit`
- 编辑区：空白
- 大纲：空

### 用例 2：打开存在的文件
- 命令：`mdedit.exe README.md`
- 结果：✓ 通过
- 标题栏：`README.md - mdedit`
- 编辑区：显示 README 内容
- 大纲：显示标题列表

### 用例 3：打开不存在的文件
- 命令：`mdedit.exe not-exist.md`
- 结果：✓ 通过
- 错误提示：弹出错误框
- 回退行为：进入空白页

### 回归测试
- Ctrl+O：✓ 通过
- Ctrl+S：✓ 通过
- Ctrl+N：✓ 通过
- 大纲跳转：✓ 通过
- WYSIWYG 渲染：✓ 通过

## 结论
所有测试用例通过，功能正常。
EOF
```

- [ ] **步骤 6：Commit 测试结果**

```bash
git add docs/superpowers/test-results/2026-05-28-cli-open-file.md
git commit -m "test: 添加命令行打开文件测试结果

记录手工测试结果，所有用例通过。

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## 任务 5：更新 README 文档

**文件：**
- 修改：`README.md:29-35`

- [ ] **步骤 1：更新运行说明**

将 `README.md:29-35` 的运行部分从：

```markdown
### 运行

\```bash
cargo run
# 或直接运行
./target/release/mdedit
\```
```

改为：

```markdown
### 运行

\```bash
# 启动空白编辑器
cargo run

# 直接打开文件
cargo run -- README.md

# 或使用编译后的可执行文件
./target/release/mdedit
./target/release/mdedit README.md
\```
```

- [ ] **步骤 2：验证 Markdown 格式**

运行：

```bash
cd /c/MyWork/AiCode/mdedit
./target/release/mdedit.exe README.md
```

预期：README.md 正常显示，无格式错误

- [ ] **步骤 3：Commit**

```bash
git add README.md
git commit -m "docs: 更新 README 运行说明

添加命令行打开文件的使用示例。

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## 规格覆盖度检查

| 规格需求 | 对应任务 | 状态 |
|---------|---------|------|
| 解析命令行参数 | 任务 1 | ✓ |
| 读取文件内容 | 任务 1 | ✓ |
| 失败时弹错误框 | 任务 1 | ✓ |
| 修改 MdEditApp::new() 签名 | 任务 2 | ✓ |
| 根据参数初始化 document | 任务 2 | ✓ |
| 连接入口与应用 | 任务 3 | ✓ |
| 手工测试验证 | 任务 4 | ✓ |
| 更新文档 | 任务 5 | ✓ |

所有规格需求已覆盖。

---

## 执行说明

本计划共 5 个任务，预计总耗时 30-40 分钟。

每个任务独立可测试，按顺序执行。任务 1-3 完成后即可进行任务 4 的手工测试。

建议使用 **subagent-driven-development** 执行，每个任务由独立子代理完成，任务间进行审查。
