use super::cell::Cell;

#[derive(Debug, Clone)]
pub struct Operation {
    pub row: usize,
    pub col: usize,
    pub old_cell: Option<Cell>,
    pub new_cell: Option<Cell>,
}

#[derive(Debug, Clone)]
pub struct UndoManager {
    pub undo_stack: Vec<Operation>,
    pub redo_stack: Vec<Operation>,
}

impl UndoManager {
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    pub fn push_operation(&mut self, op: Operation) {
        self.undo_stack.push(op);
        self.redo_stack.clear();
    }

    pub fn undo(&mut self) -> Option<Operation> {
        if let Some(op) = self.undo_stack.pop() {
            self.redo_stack.push(op.clone());
            return Some(op);
        }
        None
    }

    pub fn redo(&mut self) -> Option<Operation> {
        if let Some(op) = self.redo_stack.pop() {
            self.undo_stack.push(op.clone());
            return Some(op);
        }
        None
    }
}
