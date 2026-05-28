pub struct EditOp {
    pub offset: usize,
    pub old_text: String,
    pub new_text: String,
}

pub struct History {
    undo_stack: Vec<EditOp>,
    redo_stack: Vec<EditOp>,
}

impl History {
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    pub fn push(&mut self, op: EditOp) {
        self.undo_stack.push(op);
        self.redo_stack.clear();
    }

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
