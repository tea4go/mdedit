# mdedit 命令行打开文件设计

## 背景

当前 `mdedit` 已支持通过菜单项或快捷键 `Ctrl+O` 打开 Markdown 文件，但尚不支持在启动时通过命令行参数直接载入文件。

现状如下：

- `src/main.rs` 仅负责初始化 `eframe` 窗口并创建 `MdEditApp`
- `src/app.rs` 中的 `open_file()` 仅通过 `rfd::FileDialog::pick_file()` 选择文件
- 程序当前不解析 `std::env::args()`，因此执行 `mdedit.exe README.md` 时不会自动打开文件

这导致实际验证与日常使用存在限制：

- 无法从命令行、文件关联或外部工具直接将 Markdown 文件交给 `mdedit`
- 自动化验收时无法稳定地将 `README.md` 注入程序并检查渲染结果

## 目标

为 `mdedit` 增加启动参数打开文件能力，使其支持：

```bash
mdedit.exe README.md
```

并满足以下行为约束：

1. 启动时若提供单个文件路径，则自动尝试打开该文件
2. 文件读取成功后，编辑器直接进入该文档
3. 文件读取失败时，弹出错误提示框
4. 错误提示后，程序继续启动并回退到空白新建页
5. 未提供路径时，保持当前行为不变

## 非目标

本次不包含以下内容：

- 不支持一次打开多个文件
- 不支持目录作为启动参数
- 不支持拖拽打开功能
- 不支持最近文件恢复逻辑变更
- 不支持非 UTF-8 编码自动探测
- 不支持启动参数中的额外 CLI 选项（如 `--help`、`--version`）

## 方案对比

### 方案 A：通过 `egui_ctx.data()` 传递启动文件

在 `main.rs` 中解析命令行参数并读取文件，再通过 `CreationContext` 的 `egui_ctx.data_mut()` 暂存数据，`MdEditApp::new()` 从上下文读取。

**优点：**

- 不改 `MdEditApp::new()` 的函数签名

**缺点：**

- 数据传递链路隐蔽，不直观
- 启动参数属于应用初始化输入，放进 UI 上下文语义不清晰
- 后续维护时不容易快速定位来源

### 方案 B：在 `main.rs` 解析参数，直接传给 `MdEditApp::new()`

在 `main.rs` 中解析命令行参数并读取文件，将结果作为 `Option<(PathBuf, String)>` 直接传入 `MdEditApp::new()`。

**优点：**

- 参数传递最直观
- 初始化职责清晰，应用启动输入集中在入口层
- 改动范围小，只影响 `main.rs` 与 `app.rs`
- 与当前项目结构最一致，适合最小实现

**缺点：**

- 需要调整 `MdEditApp::new()` 的签名

### 方案 C：在 `MdEditApp` 内增加 `pending_file`，首帧加载

`main.rs` 中只传入路径，`MdEditApp` 保存待打开文件，在 `update()` 首帧中执行读取和加载。

**优点：**

- 可将部分逻辑延后到 UI 生命周期中处理

**缺点：**

- 状态字段增加，逻辑分散
- 文件加载本质上属于初始化，不应延迟到渲染阶段
- 相比本需求显得更绕

## 选型结论

采用 **方案 B**。

理由：

- 最符合当前项目的结构与复杂度
- 入口解析参数，应用构造函数接收初始化数据，职责边界自然
- 不引入额外状态字段
- 后续若要支持文件关联、最近文件恢复或多窗口，也更容易沿着入口层扩展

## 详细设计

### 1. 启动参数解析

在 `src/main.rs` 中新增启动参数解析逻辑：

- 读取 `std::env::args().nth(1)` 作为可选文件路径
- 若不存在参数，则返回 `None`
- 若存在参数，则尝试读取文件内容

返回值结构：

```rust
Option<(PathBuf, String)>
```

含义：

- `Some((path, content))`：成功读取文件
- `None`：未传参，或传参但读取失败

### 2. 读取失败处理

若传入了路径但读取失败，则在 `main.rs` 中弹出错误提示框：

- 使用 `rfd::MessageDialog`
- 文案包含路径与底层错误信息
- 提示完成后继续启动程序
- 程序进入空白新建页

错误提示格式：

```text
无法打开文件：<path>

<error>
```

这样可以覆盖三类常见失败：

- 文件不存在
- 无读取权限
- 文件不是有效 UTF-8

### 3. 应用初始化

修改 `src/app.rs` 中的构造函数签名：

```rust
pub fn new(
    cc: &eframe::CreationContext<'_>,
    initial_file: Option<(PathBuf, String)>,
) -> Self
```

初始化逻辑：

- 若 `initial_file` 为 `Some`：
  - 调用 `Document::from_file(path, content)`
  - 立即调用 `outline::extract_outline()` 生成大纲
  - 标题栏显示文件名
- 若 `initial_file` 为 `None`：
  - 保持现有 `Document::new()` 行为
  - 大纲为空

### 4. UI 行为

载入成功后，界面应满足：

- 标题栏从 `未命名 - mdedit` 变为 `<文件名> - mdedit`
- 左侧大纲面板根据文档标题即时生成
- 编辑区直接展示该 Markdown 文档内容
- WYSIWYG 块渲染与当前逻辑一致，不新增特殊分支

### 5. 对现有流程的影响

本功能不改变以下现有行为：

- `Ctrl+O` 仍通过文件对话框打开文件
- `Ctrl+S`/`Ctrl+Shift+S` 行为保持不变
- 新建文档仍默认空白页
- 程序启动时若没有参数，依旧展示空白页

## 代码改动范围

预计仅修改以下文件：

### `src/main.rs`

新增：

- `use std::env`
- `use std::fs`
- `use std::path::PathBuf`
- 启动参数解析函数
- 错误提示框逻辑
- 将 `initial_file` 传给 `MdEditApp::new()`

### `src/app.rs`

修改：

- `MdEditApp::new()` 签名
- 根据 `initial_file` 初始化 `document` 与 `outline_items`

不需要修改：

- `document/` 模块
- `editor/` 模块
- `renderer/` 模块
- `outline/` 模块

因为当前已有：

- `Document::new()`
- `Document::from_file()`
- `outline::extract_outline()`
- 标题栏自动读取 `document.path`

这些能力已经足够复用。

## 测试方案

### 手工测试

#### 用例 1：无参数启动

```bash
mdedit.exe
```

期望：

- 程序正常启动
- 标题栏显示 `未命名 - mdedit`
- 大纲为空

#### 用例 2：打开存在的 Markdown 文件

```bash
mdedit.exe README.md
```

期望：

- 程序正常启动
- 标题栏显示 `README.md - mdedit`
- 文档内容已加载
- 左侧大纲正常生成

#### 用例 3：打开不存在的文件

```bash
mdedit.exe not-exist.md
```

期望：

- 程序弹出错误提示框
- 提示框关闭后，编辑器继续启动
- 标题栏显示 `未命名 - mdedit`
- 空白页可编辑

#### 用例 4：打开非 UTF-8 文件

期望：

- 程序弹出读取失败提示
- 提示后进入空白页

### 回归检查

需确认本功能未破坏：

- 菜单栏 `文件 -> 打开`
- `Ctrl+O`
- `Ctrl+S`
- `Ctrl+Shift+S`
- 大纲渲染
- WYSIWYG 块切换

## 风险与处理

### 风险 1：相对路径解析依赖当前工作目录

命令 `mdedit.exe README.md` 使用相对路径时，实际解析结果取决于启动目录。

**处理：** 本次接受该行为，不额外改造。与大多数桌面应用一致，路径由操作系统传入后直接按当前工作目录解析。

### 风险 2：错误弹窗出现在窗口创建前

若文件读取失败，错误提示框会先于主窗口显示。

**处理：** 本次接受该行为。因为需求明确要求「弹错误后回退到空白页」，入口层处理最简单稳定。

### 风险 3：后续扩展到多文件时接口不够通用

**处理：** 当前仅支持单文件，`Option<(PathBuf, String)>` 足够。未来需要多文件时，可演进为 `Vec<OpenedFile>`。

## 实现边界

本次实现完成后，`mdedit` 将具备最基本的桌面编辑器启动行为：

- 双击或命令行传入文件可直接打开
- 打不开时有明确反馈
- 不影响现有交互路径

这是后续进行 README 界面验收与文件关联扩展的前置能力。
