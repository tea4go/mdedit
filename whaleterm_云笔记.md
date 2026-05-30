# WhaleTerminal 云笔记功能需求文档

## 1. 产品概述

云笔记是 WhaleTerminal 内置的 Markdown 编辑器模块，支持多数据源（本地文件、Gitee Gist 云端、研究云文档、WhaleBook 文档库）的笔记管理。核心能力包括：所见即所得编辑、文档锁定与冲突检测、自动保存、多源文件树管理、大纲导航、全文搜索、历史记录等。

目标：用 Rust 重新实现一个独立的 Markdown 编辑器桌面应用，功能与 WhaleTerminal 云笔记保持一致。

---

## 2. 整体架构

### 2.1 技术选型（原项目参考）

| 层级 | 原项目技术 | 说明 |
|------|-----------|------|
| 前端框架 | Vue 3 + Pinia | 状态管理 |
| 编辑器 | Vditor 3.10.7 | 所见即所得 Markdown 编辑器 |
| 代码高亮 | CodeMirror 6 | 代码块语法高亮 |
| Markdown 解析 | markdown-it 13.0 | 渲染引擎 |
| 后端 | Go (Wails 框架) | 桌面应用框架 |
| 云存储 | Gitee Gist API | 云端笔记存储 |

### 2.2 UI 布局

```
┌─────────────────────────────────────────────────────┐
│                    工具栏 (Toolbar)                    │
├──────────────┬──────────────────────────────────────┤
│  左侧面板     │         右侧编辑区                     │
│              │                                      │
│ ┌──────────┐ │  ┌────────────────────────────────┐  │
│ │ 文件树    │ │  │                                │  │
│ │ (多Tab)  │ │  │      Markdown 编辑器            │  │
│ │          │ │  │                                │  │
│ │ - 本地   │ │  │                                │  │
│ │ - Gist   │ │  │                                │  │
│ │ - 云文档  │ │  │                                │  │
│ │          │ │  │                                │  │
│ ├──────────┤ │  │                                │  │
│ │ 大纲     │ │  │                                │  │
│ │ (Outline)│ │  │                                │  │
│ ├──────────┤ │  │                                │  │
│ │ 搜索结果  │ │  │                                │  │
│ └──────────┘ │  └────────────────────────────────┘  │
├──────────────┴──────────────────────────────────────┤
│                    状态栏                             │
└─────────────────────────────────────────────────────┘
```

---

## 3. 编辑器核心功能

### 3.1 编辑模式

支持三种编辑模式，用户可切换：

| 模式 | 标识 | 说明 |
|------|------|------|
| 即时渲染 (IR) | `ir` | 所见即所得，输入即渲染（默认模式） |
| 分屏预览 (SV) | `sv` | 左侧源码，右侧实时预览 |
| 纯所见即所得 | `wysiwyg` | 类 Word 的纯可视化编辑 |

### 3.2 工具栏功能

按顺序排列的工具栏按钮：

| 功能 | Markdown 语法 | 说明 |
|------|--------------|------|
| 撤销 | - | 编辑历史回退 |
| 重做 | - | 编辑历史前进 |
| 标题选择器 | `# ~ ######` | H1-H6 下拉选择 |
| 加粗 | `**text**` | 选中文本加粗 |
| 斜体 | `*text*` | 选中文本斜体 |
| 删除线 | `~~text~~` | 选中文本删除线 |
| 链接 | `[text](url)` | 插入超链接 |
| 无序列表 | `* ` | 行首插入 |
| 有序列表 | `1. ` | 自动编号 |
| 任务列表 | `* [ ] ` | 复选框列表 |
| 增加缩进 | - | 列表层级增加 |
| 减少缩进 | - | 列表层级减少 |
| 引用 | `> ` | 块引用 |
| 分割线 | `---` | 水平分割线 |
| 代码块 | ` ``` ` | 围栏代码块 |
| 行内代码 | `` ` `` | 行内代码 |
| 表格 | Markdown 表格 | 默认插入 2x2 表格 |
| 字体颜色 | - | 颜色选择器（自定义组件） |
| 图片上传 | `![](url)` | 上传图片附件 |

### 3.3 Markdown 渲染能力

#### 基础语法
- 标题 (H1-H6)
- 段落与换行
- 粗体、斜体、删除线
- 有序/无序/任务列表
- 引用块（支持嵌套）
- 代码块（围栏式，支持语言标注）
- 行内代码
- 链接与图片
- 表格（支持对齐）
- 水平分割线
- 脚注
- 高亮标记 (mark)

#### 扩展语法
- **数学公式**: KaTeX 引擎，支持行内 `$...$` 和块级 `$$...$$`
- **Mermaid 图表**: 流程图、时序图、类图、状态图、甘特图等
- **ECharts 图表**: 嵌入式图表
- **PlantUML**: UML 图
- **Graphviz**: DOT 语言图
- **Flowchart.js**: 流程图
- **思维导图**: Mindmap
- **ABC 记谱法**: 音乐记谱
- **目录生成**: `[TOC]` 标记

#### 代码高亮
支持 50+ 编程语言语法高亮，包括但不限于：
bash, javascript, typescript, python, java, go, rust, cpp, c, csharp, ruby, php, sql, html, css, json, yaml, xml, markdown, shell 等。

代码样式可配置（github, solarized-dark 等主题）。

### 3.4 图片处理

| 功能 | 说明 |
|------|------|
| 拖拽上传 | 拖拽图片到编辑器自动上传 |
| 粘贴上传 | 从剪贴板粘贴图片 |
| 大小限制 | 单张图片最大 10MB |
| Base64 过滤 | 阻止粘贴 Base64 编码图片（防止文档膨胀） |
| 图片预览 | 点击图片放大查看，支持鼠标滚轮缩放 |
| 本地存储 | 图片存储在 `/files` 目录 |
| 云端上传 | 云文档模式下图片上传到云端 |

### 3.5 表格智能处理

- 从 Excel/电子表格粘贴数据时，自动检测并转换为 Markdown 表格
- 表格单元格宽度限制为编辑器宽度的 80%
- 支持表格对齐语法

### 3.6 编辑器辅助功能

| 功能 | 说明 |
|------|------|
| 打字机模式 | 光标始终保持在屏幕中央 |
| 自动换行 | 可切换开关 |
| 字体缩放 | Ctrl+Plus/Minus 调整字号 |
| 中英文自动空格 | 中英文之间自动添加空格 |

---

## 4. 文件管理系统

### 4.1 多数据源架构

系统支持以下笔记数据源，每个数据源在左侧面板以独立 Tab 展示：

| 数据源 | 标识 | 存储位置 | 锁定支持 |
|--------|------|---------|----------|
| 本地文件 | `local` | 本地文件系统 | 否 |
| Gitee Gist | `gist` | Gitee API 云端 | 是（机器锁） |
| 研究云文档 | `cloud` | WhaleCloud API | 是（用户锁） |
| WhaleBook | `whale` | WhaleBook API | 是（用户锁） |
| Skills 脚本 | `skills` | 本地指定目录 | 否 |
| 钉钉日志 | `dingtalk` | 钉钉 API | 否 |

### 4.2 本地文件管理

#### 目录结构
```
<数据目录>/
├── group1/          # 文件夹（分组）
│   ├── note1.md
│   └── note2.md
├── group2/
│   └── note3.md
├── files/           # 图片附件目录
│   └── xxx.png
└── ungrouped.md     # 无分组文件
```

#### 文件操作
- **新建文件**: 在指定分组下创建 .md 文件
- **新建文件夹**: 创建分组目录
- **重命名**: 文件和文件夹重命名
- **删除**: 移入回收站（非物理删除）或物理删除
- **移动**: 文件/文件夹拖拽移动到其他分组
- **复制**: 文件/文件夹复制（自动添加 `_copy` 后缀）
- **恢复已删除文件**: 从回收站恢复

#### 文件树特性
- 树形结构展示，支持展开/折叠
- 支持拖拽排序和移动
- 自然排序（数字按大小排列）
- 隐藏文件过滤（dot 前缀 + Windows 隐藏属性）
- 支持符号链接/Junction Link 解析
- 虚拟滚动优化大量文件性能
- 右键上下文菜单

### 4.3 Gist 云端文件管理

#### 存储结构
- 每个文件存储为独立的 Gitee Gist
- 使用 `QNoteFileList.json` 作为主索引文件
- 文件路径使用 UUID（实际文件名存储在元数据中）
- 版本标识: "QNote V13"

#### 索引文件格式 (QNoteFileList.json)
```json
{
  "version": "QNote V13",
  "groups": [
    {
      "name": "分组名",
      "files": [
        {
          "gistId": "gitee-gist-id",
          "name": "文件名.md",
          "groupName": "分组名",
          "filePath": "uuid-path",
          "fileSize": 1024,
          "modTime": "2026-01-01T00:00:00Z"
        }
      ]
    }
  ]
}
```

#### Gist 特性
- UTF-8 过滤：自动移除 4 字节 emoji 字符（数据库兼容）
- HTTP 超时: 60 秒
- 进度事件: 上传/下载进度回调
- 孤儿 Gist 检测: DiffCloudNote 对比本地索引与远端实际 Gist

### 4.4 跨源操作

支持不同数据源之间的文件移动和复制：

| 操作 | 说明 |
|------|------|
| 本地 → Gist | 移动/复制本地文件到云端 |
| Gist → 本地 | 移动/复制云端文件到本地 |
| 本地 → 云文档 | 粘贴本地文件到研究云（含图片上传） |
| Gist → 云文档 | 粘贴 Gist 文件到研究云 |
| 聊天消息 → 本地 | 从聊天消息创建本地笔记 |
| 聊天消息 → 云文档 | 从聊天消息创建云文档 |
| 导出全部 Gist | 批量导出所有 Gist 文档到本地 |

---

## 5. 文档锁定与冲突检测

### 5.1 Gist 文档锁定机制

#### 锁定信息结构
```
TDocLockInfo {
  Lock: bool          // 是否锁定
  LockId: string      // 锁唯一标识
  LockHost: string    // 锁定机器 ID
  ExpiresAt: time     // 过期时间（30分钟）
  IsOwnLock: bool     // 是否本机锁定
}
```

#### 锁定流程
1. 用户进入编辑模式 → 尝试获取锁
2. 获取成功 → `gistEditMode = 'locked'`，开始编辑
3. 获取失败（他人持有）→ `gistEditMode = 'normal'`，无锁编辑
4. 编辑过程中定期续锁（防止过期）
5. 保存时：
   - locked 模式 + 继续编辑 → 保存并续锁
   - locked 模式 + 结束编辑 → 保存并释放锁
   - normal 模式 → 直接保存（无锁验证）
6. 锁过期自动释放：倒计时到期后自动保存并释放

#### 锁定超时
- 锁有效期: 30 分钟
- 前端倒计时显示剩余时间
- 到期前可续锁
- 到期后自动保存并释放

### 5.2 冲突检测机制

#### 校验和对比
- 打开/读取文档时计算内容 checksum → `editBaseChecksum`
- 保存前重新读取最新内容计算 checksum
- 对比两个 checksum：
  - 相同 → 无冲突，正常保存
  - 不同 → 检测到冲突

#### 冲突处理策略
1. **文档被他人修改**: 自动另存为新文件（文件名追加时间戳）
   - 格式: `原文件名-20260530_143022.md`
2. **文档已被删除**: 自动重新创建文件并保存
3. **另存后处理**: 更新当前文件指向新文件，通知用户

### 5.3 WhaleCloud/WhaleBook 文档锁定

- 使用服务端锁机制（非本地锁）
- `deferred` 标记: 当他人持有锁时设为 true
- 显示锁持有者信息
- 锁定后刷新文档内容确保最新

---

## 6. 自动保存

### 6.1 自动保存逻辑

- **触发条件**: 用户停止输入 N 秒后触发（N 可配置，最小 60 秒）
- **开关**: 用户可在设置中开启/关闭自动保存
- **静默保存**: 自动保存不弹出提示
- **保存队列**: 如果上一次保存尚未完成，复用进行中的保存任务

### 6.2 保存流程

```
用户编辑 → 等待 N 秒无操作 → 触发自动保存
                                    ↓
                          检查是否有未完成的保存任务
                                    ↓ (无)
                          同步编辑器内容（SV 模式需同步）
                                    ↓
                          冲突检测（对比 checksum）
                                    ↓ (无冲突)
                          根据数据源路由保存
                          ├── 本地 → 写入文件系统
                          ├── Gist → 调用 Gitee API（带锁验证）
                          └── 云文档 → 调用云端 API
                                    ↓
                          更新 editBaseChecksum
                          设置 ifModify = false
```

### 6.3 应用退出时保存

- 退出前检查是否有未保存修改
- Gist 文档: 自动保存并释放锁
- 本地文档: 根据用户偏好决定（自动保存 / 弹窗确认）
- 钉钉文档: 自动保存草稿

---

## 7. 大纲导航 (Outline)

### 7.1 功能特性

| 功能 | 说明 |
|------|------|
| 自动生成 | 从 Markdown 标题自动提取生成目录树 |
| 层级展开 | 可配置展开到第 N 级（1-3） |
| 自动编号 | 可选开启，支持多种编号格式（点号/无/逗号） |
| 点击跳转 | 点击大纲项跳转到对应标题位置 |
| 滚动同步 | 编辑器滚动时高亮当前所在的大纲项 |
| 右键菜单 | 大纲项的上下文操作 |

### 7.2 编号格式

| 格式 | 示例 |
|------|------|
| 点号 (dot) | 1. / 1.1 / 1.1.1 |
| 无 (none) | 无编号 |
| 逗号 (comma) | 1, / 1,1 / 1,1,1 |

---

## 8. 搜索功能

### 8.1 编辑器内搜索 (SearchBar)

查找替换功能，类似 VS Code 的搜索栏：

| 功能 | 说明 |
|------|------|
| 实时搜索 | 输入即搜索（300ms 防抖） |
| 大小写敏感 | 可切换 |
| 全词匹配 | 可切换 |
| 上/下导航 | 在匹配项之间跳转 |
| 匹配计数 | 显示 "当前/总数" |
| 替换 | 仅编辑模式可用 |
| 全部替换 | 一键替换所有匹配 |
| 快捷键 | Ctrl+F 打开，Escape 关闭，Enter 下一个 |

#### 高亮实现
- IR/WYSIWYG 模式: 使用 mark.js 进行 DOM 高亮
- SV 模式: 使用 CodeMirror Decoration 高亮

### 8.2 全文搜索 (SearchTree)

跨文件内容搜索，结果在左侧面板展示：

| 功能 | 说明 |
|------|------|
| 搜索范围 | 所有本地 Markdown 文件 |
| 搜索模式 | 关键词 / 正则表达式 / 表达式 |
| 结果展示 | 树形结构（文件 → 匹配行） |
| 结果高亮 | 匹配文本高亮显示 |
| 点击定位 | 点击结果打开文件并跳转到匹配位置 |
| 状态追踪 | IDLE → RENDER_WORKING → RENDER_DONE → LOCATE_WORKING |

---

## 9. 历史记录

### 9.1 功能特性

| 功能 | 说明 |
|------|------|
| 自动记录 | 打开文件时自动添加到历史 |
| 最大条数 | 非置顶项最多 20 条 |
| 置顶功能 | 重要文件可置顶，不受条数限制 |
| 恢复打开 | 点击历史项恢复打开文件 |
| 滚动位置 | 记录并恢复文档滚动位置 |
| 编辑模式 | 记录并恢复编辑模式 |
| 清空历史 | 清空所有非置顶项 |
| 模块过滤 | 根据数据源可见性过滤历史列表 |

### 9.2 历史项数据结构

```json
{
  "filePath": "path/to/file.md",
  "fileName": "file.md",
  "groupName": "group1",
  "idKey": "local|gist|skills|dingtalk",
  "gistId": "xxx",
  "scrollPosition": 120,
  "editMode": "ir",
  "pinned": false,
  "timestamp": 1717000000000
}
```

### 9.3 过滤规则

- 钉钉文档始终排除
- 未登录 Gist 时隐藏 Gist 历史
- 未登录云端时隐藏云文档历史
- 用户可配置各模块可见性

---

## 10. 预览服务器

### 10.1 功能

内置本地 HTTP 预览服务器，用于在浏览器中预览 Markdown 渲染结果。

| 功能 | 说明 |
|------|------|
| 预览模式 | index（目录）/ single（单文件）/ pure（纯内容） |
| 端口 | 可配置（默认 34118，排除 34116-34117） |
| 图片服务 | 独立图片服务端口 34117 |
| 分享 | 生成分享链接，复制到剪贴板并打开浏览器 |
| 主题 | 预览支持主题定制 |
| 自动启动 | 可配置是否随应用启动 |

---

## 11. 导出功能

### 11.1 支持的导出操作

| 操作 | 说明 |
|------|------|
| 导出全部 Gist | 批量导出所有云端笔记到本地 |
| 导出云文档 | 导出研究云文档到本地 |
| 导出本地文件 | 导出本地文件（打包） |
| 表格导出 XLSX | 将 Markdown 表格导出为 Excel 文件 |

### 11.2 导出特性

- 实时进度显示
- 事件驱动架构（progress / complete / error 事件）
- 多实例并行导出
- 完成后显示文件位置，支持点击打开

---

## 12. 文件预览（非 Markdown）

支持预览以下格式的文件（只读）：

| 格式 | 说明 |
|------|------|
| PDF | 内嵌 PDF 阅读器 |
| Excel | 支持多 Sheet 切换、单元格选择 |
| PowerPoint | 幻灯片预览 |
| Word | 文档预览 |

通用功能：缩放控制（放大/缩小/重置）、鼠标滚轮缩放、加载状态指示。

---

## 13. 键盘快捷键

### 13.1 编辑器快捷键

| 快捷键 (Win) | 快捷键 (Mac) | 功能 |
|-------------|-------------|------|
| Ctrl+E | Cmd+E | 切换编辑模式 |
| Ctrl+F | Cmd+F | 打开搜索栏 |
| Ctrl+S | Cmd+S | 保存文件 |
| Ctrl+Z | Cmd+Z | 撤销 |
| Ctrl+Y | Cmd+Y | 重做 |
| Ctrl+X | Cmd+X | 剪切 |
| Ctrl+C | Cmd+C | 复制 |
| Ctrl+V | Cmd+V | 粘贴 |
| Ctrl+A | Cmd+A | 全选 |
| Ctrl+Plus | Cmd+Plus | 字体放大 |
| Ctrl+Minus | Cmd+Minus | 字体缩小 |
| Escape | Escape | 关闭搜索栏 |

### 13.2 快捷键库

使用 Mousetrap 库实现，支持平台感知（自动适配 Win/Mac）。

---

## 14. 设置与偏好

### 14.1 可配置项

| 设置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| 字体 | 下拉选择 | 系统默认 | 编辑器字体 |
| 字号 | 数字 (12-32) | 14px | 编辑器字号 |
| 字体加粗 | 开关 | 关 | 是否加粗显示 |
| 文件列表模式 | 选择 | 树形 | 树形/平铺 |
| 数据存储目录 | 路径选择 | - | 本地 Markdown 文件存储路径 |
| 自动保存 | 开关 | 开 | 是否启用自动保存 |
| 自动保存间隔 | 数字 | 60s | 最小 60 秒 |
| 显示大纲编号 | 开关 | 关 | 大纲是否显示编号 |
| 编号格式 | 选择 | dot | dot/none/comma |
| 预览自动启动 | 开关 | 关 | 是否自动启动预览服务器 |
| 预览端口 | 数字 | 34118 | 1-65535，排除保留端口 |

### 14.2 持久化

设置通过 preference store 持久化到本地存储，应用重启后恢复。

---

## 15. 数据源可见性控制

用户可独立控制每个数据源模块的显示/隐藏：

```json
{
  "local": true,
  "gist": true,
  "cloud": true,
  "whale": true,
  "skills": true,
  "dingtalk": true
}
```

隐藏的模块不在左侧面板显示 Tab，对应的历史记录也会被过滤。

---

## 16. 文件树右键菜单

### 16.1 本地文件树

| 菜单项 | 说明 |
|--------|------|
| 新建文件 | 在当前分组下创建 .md 文件 |
| 新建文件夹 | 创建新分组 |
| 重命名 | 重命名文件/文件夹 |
| 删除 | 删除文件/文件夹 |
| 复制 | 复制文件/文件夹 |
| 移动到... | 移动到其他分组 |
| 复制到 Gist | 复制到云端 |
| 移动到 Gist | 移动到云端 |
| 在文件管理器中打开 | 打开所在目录 |

### 16.2 Gist 文件树

| 菜单项 | 说明 |
|--------|------|
| 新建文件 | 创建新 Gist 文件 |
| 新建文件夹 | 创建新分组 |
| 重命名 | 重命名 |
| 删除 | 删除 |
| 复制到本地 | 复制到本地文件系统 |
| 移动到本地 | 移动到本地文件系统 |

---

## 17. 拖拽操作

### 17.1 文件树拖拽

| 操作 | 规则 |
|------|------|
| 文件 → 文件夹 | 允许，移动文件到目标文件夹 |
| 文件夹 → 文件夹 | 允许，移动文件夹到目标文件夹内 |
| 文件 → 文件 | 不允许（不能放入非文件夹节点） |
| 只读位置 | 不允许拖入只读位置 |

### 17.2 拖拽事件

- `dragStart`: 记录拖拽源节点
- `dragOver`: 验证放置位置合法性
- `dragEnd`: 清理状态
- `drop`: 执行移动/重组操作

---

## 18. 事件系统

系统使用事件总线进行组件间通信：

### 18.1 文件操作事件

| 事件名 | 说明 |
|--------|------|
| `select-file` | 文件被选中/打开 |
| `add-md-file-success` | 本地文件/文件夹创建成功 |
| `add-gist-success` | Gist 文件/文件夹创建成功 |
| `add-cloud-success` | 云文档创建成功 |
| `add-whale-success` | WhaleBook 文档创建成功 |

### 18.2 编辑器事件

| 事件名 | 说明 |
|--------|------|
| `sync-save-content` | SV 模式同步内容 |
| `save-gist-file` | Gist 文件保存完成 |
| `clear-vditor` | 清空编辑器 |
| `restore-scroll-position` | 恢复滚动位置 |

### 18.3 锁定事件

| 事件名 | 说明 |
|--------|------|
| `lock-extended` | 锁已续期 |
| `lock-timer-updated` | 锁倒计时更新（剩余秒数） |

### 18.4 导出事件

| 事件名 | 说明 |
|--------|------|
| `export-progress` | 导出进度更新 |
| `export-complete` | 导出完成 |
| `export-error` | 导出错误 |

---

## 19. 状态管理核心数据结构

### 19.1 当前文件状态

```typescript
interface CurrentFile {
  filePath: string;       // 文件路径
  fileName: string;       // 文件名
  groupName: string;      // 所属分组
  idKey?: 'gist' | 'skills' | 'dingtalk';  // 数据源标识
  gistId?: string;        // Gist ID（仅 Gist 文件）
  docId?: string;         // 文档 ID（仅云文档）
  isWhaleBook?: boolean;  // 是否 WhaleBook 文档
  bookId?: string;        // 所属文档库 ID
}
```

### 19.2 编辑器状态

```typescript
interface EditorState {
  curMdfile: CurrentFile;           // 当前打开的文件
  curMdfileContent: string;         // 当前文件内容
  isEditMode: boolean;              // 是否处于编辑模式
  editMode: 'ir' | 'sv' | 'wysiwyg'; // 编辑模式类型
  ifModify: boolean;                // 是否有未保存修改
  editBaseChecksum: string;         // 编辑前内容校验和
  contentEditor: EditorInstance;    // 编辑器实例引用
  showOutline: boolean;             // 是否显示大纲
  showSearchBar: boolean;           // 是否显示搜索栏
  gistEditMode: 'locked' | 'normal' | null; // Gist 编辑锁模式
  previewServerRunning: boolean;    // 预览服务器是否运行
}
```

---

## 20. API 接口清单

### 20.1 本地文件操作 API

| 方法 | 参数 | 返回 | 说明 |
|------|------|------|------|
| LoadMarkDownFiles() | - | FileTree | 加载本地文件树 |
| ReadMarkDownFile(path) | 文件路径 | base64 内容 | 读取文件 |
| WriteMarkDownFile(path, content) | 路径, 内容 | bool | 写入文件 |
| AddMDFile(name, group) | 文件名, 分组 | string | 创建文件 |
| RenameMDFile(old, new, group) | 旧名, 新名, 分组 | bool | 重命名 |
| DeleteMDFile(name, group) | 文件名, 分组 | bool | 删除文件 |
| RestoreDeletedMDFile(name, group) | 文件名, 分组 | bool | 恢复删除 |
| AddGroup(name, parent) | 分组名, 父路径 | bool | 创建分组 |
| RenameGroup(old, new) | 旧名, 新名 | bool | 重命名分组 |
| DeleteGroup(name) | 分组名 | bool | 删除分组 |
| MoveMDFile(from, to, name) | 源, 目标, 文件名 | bool | 移动文件 |
| MoveGroup(from, to) | 源, 目标 | bool | 移动分组 |
| CopyMDFile(from, to, name) | 源, 目标, 文件名 | bool | 复制文件 |
| CopyGroup(from, to) | 源, 目标 | bool | 复制分组 |
| SearchFileContent(query, mode) | 搜索词, 模式 | Results | 全文搜索 |

### 20.2 Gist 云端 API

| 方法 | 参数 | 返回 | 说明 |
|------|------|------|------|
| LoadMarkDownCloudFiles() | - | FileTree | 加载云端文件树 |
| TryReadMarkDownFile(gistId, path) | ID, 路径 | 内容+锁信息 | 尝试读取（含锁检测） |
| ReadMarkDownFile(gistId, path) | ID, 路径 | 内容 | 读取文件 |
| WriteMarkDownFile(gistId, path, content, lock) | ID, 路径, 内容, 是否带锁 | bool | 写入文件 |
| ExtendDocumentLock(gistId, path) | ID, 路径 | LockInfo | 续锁 |
| ReleaseDocumentLock(gistId, path) | ID, 路径 | bool | 释放锁 |
| GetDocumentLockStatus(gistId, path) | ID, 路径 | LockInfo | 查询锁状态 |
| GetSheetAsXLSX(content) | 表格内容 | xlsx 文件 | 导出 Excel |

### 20.3 WhaleCloud/WhaleBook API

| 方法 | 说明 |
|------|------|
| getBookDetail(bookId) | 获取文档库详情 |
| getDocInfo(docId) | 获取文档信息 |
| saveDoc(docId, content) | 保存文档 |
| lockDoc(docId) | 锁定文档 |
| unlockDoc(docId) | 解锁文档 |
| addBook(name) | 创建文档库 |
| delBook(bookId) | 删除文档库 |
| copyFile(params) | 复制文件 |
| moveFile(params) | 移动文件 |
| renameFolderAndFile(params) | 重命名 |
| attachImg(bookId, docId, file) | 上传图片 |
| getWhaleBookList() | 获取文档库列表 |
| getBookStacks(bookId) | 获取文档库目录结构 |

---

## 21. 特殊功能

### 21.1 Skills 脚本笔记

- 支持指定多个本地目录作为 Skills 数据源
- 支持 YAML Front Matter 的 skill.md 文件
- 编辑时将 YAML 头部转换为 Markdown 表格显示
- 保存时将表格转回 YAML 格式
- 支持 skill 描述检测和目录刷新

### 21.2 粘贴智能处理

| 粘贴内容 | 处理方式 |
|---------|---------|
| 纯文本 | 直接插入 |
| HTML 表格 | 转换为 Markdown 表格 |
| Excel 数据 | 检测并转换为 Markdown 表格 |
| 图片文件 | 上传并插入图片链接 |
| Base64 图片 | 阻止粘贴（防止文档膨胀） |
| 大文件 (>5MB) | 优化处理流程 |

### 21.3 机器标识

- 使用 `machineid.ProtectedID()` 或主机名作为机器唯一标识
- 用于文档锁定的机器识别
- 确保同一用户在不同设备上的锁互斥

---

## 22. 非功能性需求

### 22.1 性能要求

| 指标 | 要求 |
|------|------|
| 文件打开 | < 500ms（本地文件） |
| 编辑响应 | 实时，无感知延迟 |
| 自动保存 | 后台静默，不阻塞编辑 |
| 文件树加载 | 支持虚拟滚动，大量文件不卡顿 |
| 搜索响应 | < 2s（本地全文搜索） |

### 22.2 数据安全

| 要求 | 说明 |
|------|------|
| 防丢失 | 冲突检测 + 自动另存 |
| 锁机制 | 防止多端同时编辑覆盖 |
| 退出保护 | 退出前检查未保存内容 |
| 回收站 | 删除文件可恢复 |

### 22.3 兼容性

- 跨平台: Windows / macOS / Linux
- 文件编码: UTF-8
- 换行符: 自动适配平台

---

## 23. Rust 实现建议

### 23.1 推荐技术栈

| 组件 | 推荐方案 | 说明 |
|------|---------|------|
| 桌面框架 | Tauri 2.0 | Rust 后端 + Web 前端 |
| 前端框架 | Vue 3 / Solid / Svelte | 保持与原项目一致可选 Vue |
| Markdown 编辑器 | Milkdown / Tiptap / Vditor | 推荐 Milkdown（Rust 生态友好） |
| 代码高亮 | Shiki / Prism | |
| 数学公式 | KaTeX | |
| 图表 | Mermaid.js | |
| 文件监听 | notify crate | 文件系统变更监听 |
| HTTP 客户端 | reqwest | 云端 API 调用 |
| 序列化 | serde + serde_json | |
| 数据库 | SQLite (rusqlite) | 本地索引和设置存储 |

### 23.2 核心模块划分

```
src/
├── main.rs              # 应用入口
├── editor/              # 编辑器核心
│   ├── mod.rs
│   ├── document.rs      # 文档模型
│   ├── history.rs       # 编辑历史（undo/redo）
│   └── checksum.rs      # 内容校验
├── storage/             # 存储层
│   ├── mod.rs
│   ├── local.rs         # 本地文件操作
│   ├── gist.rs          # Gitee Gist API
│   ├── cloud.rs         # WhaleCloud API
│   └── index.rs         # 文件索引管理
├── lock/                # 文档锁定
│   ├── mod.rs
│   ├── machine_id.rs    # 机器标识
│   └── lock_manager.rs  # 锁生命周期管理
├── search/              # 搜索引擎
│   ├── mod.rs
│   ├── content.rs       # 全文搜索
│   └── regex.rs         # 正则搜索
├── preview/             # 预览服务器
│   ├── mod.rs
│   └── server.rs        # HTTP 预览服务
├── export/              # 导出功能
│   ├── mod.rs
│   └── xlsx.rs          # Excel 导出
├── config/              # 配置管理
│   ├── mod.rs
│   └── preferences.rs   # 用户偏好
└── ui/                  # 前端资源
    └── ...              # Vue/Web 前端代码
```

### 23.3 关键实现注意事项

1. **文档锁定**: 使用 Mutex 保护并发操作，锁信息持久化到云端
2. **自动保存**: 使用 tokio 定时器实现，保存操作异步执行
3. **文件监听**: 使用 notify crate 监听本地文件变更，实时更新文件树
4. **UTF-8 过滤**: 上传云端前过滤 4 字节字符
5. **进度回调**: 使用 Tauri 事件系统向前端推送进度
6. **虚拟滚动**: 前端实现，大文件树不全量渲染

---

## 24. 术语表

| 术语 | 说明 |
|------|------|
| IR 模式 | Instant Render，即时渲染模式 |
| SV 模式 | Split View，分屏预览模式 |
| Gist | Gitee 代码片段服务，用作云端存储 |
| WhaleCloud | 研究云文档平台 |
| WhaleBook | 鲸书文档库平台 |
| editBaseChecksum | 编辑前内容的哈希值，用于冲突检测 |
| deferred | 文档锁被他人持有的状态标记 |
| Skills | 脚本/技能文件，支持 YAML Front Matter |
| QNoteFileList.json | Gist 云端笔记的主索引文件 |
