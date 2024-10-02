use std::ops::Add;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Cursor {
    row: usize,
    col: usize,
    snapback_col: usize,
    // offsets are used for commands that visually should move the cursor but shouldn't influence in
    // the actual cursor position on the content
    col_offset: usize,
    row_offset: usize,
}

impl Cursor {
    pub fn move_left(&mut self, amount: usize) {
        self.col = self.col.saturating_sub(amount);
        self.snapback_col = self.col;
    }

    pub fn move_down(&mut self, amount: usize) {
        self.row = self.row.add(amount);
    }

    pub fn move_up(&mut self, amount: usize) {
        self.row = self.row.saturating_sub(amount);
    }

    pub fn move_right(&mut self, amount: usize) {
        self.col = self.col.add(amount);
        self.snapback_col = self.col;
    }

    pub fn move_to_newline_start(&mut self) {
        self.col = 0;
        self.snapback_col = 0;
        self.row = self.row.add(1);
    }

    pub fn move_to_line_start(&mut self) {
        self.col = 0;
        self.snapback_col = 0;
    }

    pub fn move_to_line_end(&mut self, line_len: usize) {
        self.col = line_len + 1;
        self.snapback_col = self.col;
    }

    pub fn move_to(&mut self, col: usize, row: usize) {
        self.move_to_row(row);
        self.move_to_col(col);
    }

    pub fn move_to_col(&mut self, col: usize) {
        self.col = col;
        self.snapback_col = col;
    }

    pub fn move_to_row(&mut self, row: usize) {
        self.row = row;
    }

    pub fn row(&self) -> usize {
        self.row
    }

    pub fn col(&self) -> usize {
        self.col
    }

    pub fn row_with_offset(&self) -> usize {
        self.row + self.row_offset
    }

    pub fn col_with_offset(&self) -> usize {
        self.col + self.col_offset
    }

    pub fn set_col_offset(&mut self, offset: usize) {
        self.col_offset = offset;
    }

    pub fn set_row_offset(&mut self, offset: usize) {
        self.row_offset = offset;
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
        match line_len.cmp(&self.col) {
            std::cmp::Ordering::Less => {
                self.col = line_len + 1;
            }
            std::cmp::Ordering::Greater => self.col = self.snapback_col.min(line_len.saturating_sub(1)),
            // if both expand_col and col are the same we dont have to do nothing
            std::cmp::Ordering::Equal => {}
        }
    }
}
