/// 编辑操作记录 - 描述一次文本变更
///
/// offset: 变更在文档中的字节偏移量
/// old_text: 被替换的原始文本（用于撤销）
/// new_text: 替换后的新文本（用于重做）
pub struct EditOp {
    pub offset: usize,
    pub old_text: String,
    pub new_text: String,
}

/// 编辑历史管理器 - 支持撤销/重做
///
/// 使用双栈（undo_stack + redo_stack）实现，
/// push 时清空 redo 栈，撤销时反向操作压入 redo 栈。
pub struct History {
    /// 撤销栈：记录可撤销的操作
    undo_stack: Vec<EditOp>,
    /// 重做栈：记录可重做的操作
    redo_stack: Vec<EditOp>,
}

impl History {
    /// 创建空的编辑历史
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    /// 记录一次新的编辑操作，同时清空重做栈
    pub fn push(&mut self, op: EditOp) {
        self.undo_stack.push(op);
        self.redo_stack.clear(); // 新编辑后重做历史失效
    }

    /// 查看撤销栈顶操作（不弹出），同时生成反向操作压入重做栈
    pub fn undo(&mut self) -> Option<&EditOp> {
        self.undo_stack.last().map(|op| {
            let reverse = EditOp {
                offset: op.offset,
                old_text: op.new_text.clone(),
                new_text: op.old_text.clone(),
            };
            self.redo_stack.push(reverse);
            op
        })
    }

    /// 弹出撤销栈顶操作并生成反向操作压入重做栈
    pub fn pop_undo(&mut self) -> Option<EditOp> {
        let op = self.undo_stack.pop()?;
        let reverse = EditOp {
            offset: op.offset,
            old_text: op.new_text.clone(),
            new_text: op.old_text.clone(),
        };
        self.redo_stack.push(reverse);
        Some(op)
    }

    /// 弹出重做栈顶操作并生成反向操作压入撤销栈
    pub fn pop_redo(&mut self) -> Option<EditOp> {
        let op = self.redo_stack.pop()?;
        let reverse = EditOp {
            offset: op.offset,
            old_text: op.new_text.clone(),
            new_text: op.old_text.clone(),
        };
        self.undo_stack.push(reverse);
        Some(op)
    }
}
