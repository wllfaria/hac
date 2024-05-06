use std::ops::Add;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Cursor {
    // row/col, this is where the cursor is displayed onscreen, we use these fields to determine
    // where to do text operations.
    row: usize,
    col: usize,
    // expand row/col are used to store where the cursor was when moving to a smaller line, so we
    // can restore it to the position it was if the line length allows
    expand_row: usize,
    expand_col: usize,
}

impl Cursor {
    pub fn move_left(&mut self, amount: usize) {
        self.col = self.col.saturating_sub(amount);
        self.expand_col = self.expand_col.saturating_sub(amount);
    }

    pub fn move_down(&mut self, amount: usize) {
        self.row = self.row.add(amount);
        self.expand_row = self.expand_row.add(amount);
    }

    pub fn move_up(&mut self, amount: usize) {
        self.row = self.row.saturating_sub(amount);
        self.expand_row = self.expand_row.saturating_sub(amount);
    }

    pub fn move_right(&mut self, amount: usize) {
        self.col = self.col.add(amount);
        self.expand_col = self.expand_col.add(amount);
    }

    pub fn move_to_newline_start(&mut self) {
        self.col = 0;
        self.expand_col = 0;
        self.row = self.row.add(1);
        self.expand_row = self.expand_row.add(1);
    }

    pub fn move_to_col(&mut self, col: usize) {
        self.col = col;
        self.expand_col = col;
    }

    pub fn row(&self) -> usize {
        self.row
    }

    pub fn col(&self) -> usize {
        self.col
    }

    pub fn readable_position(&self) -> (usize, usize) {
        (self.col.add(1), self.row.add(1))
    }
}
