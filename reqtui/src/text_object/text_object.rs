use std::ops::Sub;

use crate::text_object::cursor::Cursor;
use ropey::Rope;

#[derive(Debug, Clone, PartialEq)]
pub struct Readonly;
#[derive(Debug, Clone, PartialEq)]
pub struct Write;

#[derive(Debug, Clone, PartialEq)]
pub struct TextObject<State = Readonly> {
    content: Rope,
    state: std::marker::PhantomData<State>,
}

impl<State> Default for TextObject<State> {
    fn default() -> Self {
        let content = String::default();

        TextObject {
            content: Rope::from_str(&content),
            state: std::marker::PhantomData,
        }
    }
}

impl TextObject<Readonly> {
    pub fn from(content: &str) -> TextObject<Readonly> {
        let content = Rope::from_str(content);
        TextObject::<Readonly> {
            content,
            state: std::marker::PhantomData::<Readonly>,
        }
    }

    pub fn with_write(self) -> TextObject<Write> {
        TextObject::<Write> {
            content: self.content,
            state: std::marker::PhantomData,
        }
    }
}

impl TextObject<Write> {
    pub fn insert_char(&mut self, c: char, cursor: &Cursor) {
        let line = self.content.line_to_char(cursor.row());
        let col_offset = line + cursor.col();
        self.content.insert_char(col_offset, c);
    }

    pub fn erase_previous_char(&mut self, cursor: &Cursor) {
        let line = self.content.line_to_char(cursor.row());
        let col_offset = line + cursor.col();
        self.content
            .try_remove(col_offset.saturating_sub(1)..col_offset)
            .ok();
    }

    pub fn current_line(&self, cursor: &Cursor) -> Option<&str> {
        self.content.line(cursor.row()).as_str()
    }

    pub fn len_lines(&self) -> usize {
        self.content.len_lines()
    }
}

impl<State> std::fmt::Display for TextObject<State> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.content.to_string())
    }
}
