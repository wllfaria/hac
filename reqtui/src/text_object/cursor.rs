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
        self.expand_col = self.col;
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
        self.expand_col = self.col;
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

    // when moving horizontally, expand_col and col will always have the same value.
    //
    // when moving into a smaller line (line_len < cursor.col) we make so the col is
    // equal to the length of that line;
    //
    // when moving into a bigger line (line_len > cursor.col) we make the col snap back
    // to the min col between line_len and the current expand_col position.
    pub fn maybe_snap_to_col(&mut self, line_len: usize) {
        match line_len.saturating_sub(1).cmp(&self.col) {
            std::cmp::Ordering::Less => {
                if self.expand_col.eq(&self.col) {
                    self.expand_col = self.col;
                }
                self.col = line_len.saturating_sub(1);
            }
            std::cmp::Ordering::Greater => {
                self.col = self.expand_col.min(line_len.saturating_sub(1))
            }
            // if both expand_col and col are the same we dont have to do nothing
            std::cmp::Ordering::Equal => {}
        }
    }
}
