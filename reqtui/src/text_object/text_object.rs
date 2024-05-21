use std::{
    collections::HashMap,
    ops::{Add, Sub},
};

use crate::{syntax::highlighter::Highlighter, text_object::cursor::Cursor};
use ropey::Rope;
use tree_sitter::Tree;

#[derive(Debug, PartialEq, Clone)]
pub enum LineBreak {
    Lf,
    Crlf,
}

impl From<LineBreak> for usize {
    fn from(value: LineBreak) -> usize {
        match value {
            LineBreak::Lf => 1,
            LineBreak::Crlf => 2,
        }
    }
}

impl std::fmt::Display for LineBreak {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Lf => f.write_str("\n"),
            Self::Crlf => f.write_str("\r\n"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Readonly;
#[derive(Debug, Clone, PartialEq)]
pub struct Write;

#[derive(Debug, Clone, PartialEq)]
pub struct TextObject<State = Readonly> {
    content: Rope,
    state: std::marker::PhantomData<State>,
    line_break: LineBreak,
}

impl<State> Default for TextObject<State> {
    fn default() -> Self {
        let content = String::default();

        TextObject {
            content: Rope::from_str(&content),
            state: std::marker::PhantomData,
            line_break: LineBreak::Lf,
        }
    }
}

impl TextObject<Readonly> {
    pub fn from(content: &str) -> TextObject<Readonly> {
        let content = Rope::from_str(content);
        let line_break = match content.line(0).to_string().contains("\r\n") {
            true => LineBreak::Crlf,
            false => LineBreak::Lf,
        };
        TextObject::<Readonly> {
            content,
            state: std::marker::PhantomData::<Readonly>,
            line_break,
        }
    }

    pub fn with_write(self) -> TextObject<Write> {
        TextObject::<Write> {
            content: self.content,
            state: std::marker::PhantomData,
            line_break: self.line_break,
        }
    }
}

impl TextObject<Write> {
    pub fn insert_char(&mut self, c: char, cursor: &Cursor) {
        let line = self.content.line_to_char(cursor.row());
        let col_offset = line + cursor.col();
        self.content.insert_char(col_offset, c);
    }

    pub fn insert_newline(&mut self, cursor: &Cursor) {
        let line = self.content.line_to_char(cursor.row());
        let col_offset = line + cursor.col();
        self.content
            .insert(col_offset, &self.line_break.to_string());
    }

    pub fn erase_backwards_up_to_line_start(&mut self, cursor: &Cursor) {
        if cursor.col().eq(&0) {
            return;
        }
        let line = self.content.line_to_char(cursor.row());
        let col_offset = line + cursor.col();
        self.content
            .try_remove(col_offset.saturating_sub(1)..col_offset)
            .ok();
    }

    pub fn erase_previous_char(&mut self, cursor: &Cursor) {
        let line = self.content.line_to_char(cursor.row());
        let col_offset = line + cursor.col();
        self.content
            .try_remove(col_offset.saturating_sub(1)..col_offset)
            .ok();
    }

    pub fn erase_current_char(&mut self, cursor: &Cursor) {
        let line = self.content.line_to_char(cursor.row());
        let col_offset = line + cursor.col();
        self.content.try_remove(col_offset..col_offset.add(1)).ok();
    }

    pub fn current_line(&self, cursor: &Cursor) -> Option<&str> {
        self.content.line(cursor.row()).as_str()
    }

    pub fn line_len_with_linebreak(&self, line: usize) -> usize {
        self.content
            .line(line)
            .as_str()
            .map(|line| line.len())
            .unwrap_or_default()
    }

    pub fn line_len(&self, line: usize) -> usize {
        self.content
            .line(line)
            .as_str()
            .map(|line| line.len().saturating_sub(self.line_break.clone().into()))
            .unwrap_or_default()
    }

    pub fn erase_until_eol(&mut self, cursor: &Cursor) {
        let line = self.content.line_to_char(cursor.row());
        let next_line = self.content.line_to_char(cursor.row().add(1));
        let col_offset = line + cursor.col();
        self.content
            .try_remove(col_offset..next_line.saturating_sub(1))
            .ok();
    }

    pub fn find_char_after_whitespace(&self, cursor: &Cursor) -> (usize, usize) {
        let line = self.content.line_to_char(cursor.row());
        let col_offset = line + cursor.col();
        let mut end_idx = 0;
        let mut found = false;

        for char in self.content.chars_at(col_offset) {
            match (char, found) {
                (c, false) if c.is_whitespace() => {
                    found = true;
                    end_idx = end_idx.add(1);
                }
                (c, true) if !c.is_whitespace() => break,
                _ => end_idx = end_idx.add(1),
            }
        }
        let curr_idx = col_offset.add(end_idx);
        let curr_row = self.content.char_to_line(col_offset.add(end_idx));
        let curr_row_start = self.content.line_to_char(curr_row);
        let curr_col = curr_idx.sub(curr_row_start);
        (curr_col, curr_row)
    }

    pub fn find_char_before_whitespace(&self, cursor: &Cursor) -> (usize, usize) {
        let line = self.content.line_to_char(cursor.row());
        let col_offset = line + cursor.col();
        let mut found = false;
        let mut index = col_offset.saturating_sub(1);

        for _ in (0..col_offset.saturating_sub(1)).rev() {
            let char = self.content.char(index);
            match (char, found) {
                (c, false) if c.is_whitespace() => found = true,
                (c, true) if !c.is_whitespace() => break,
                _ => {}
            }
            index = index.saturating_sub(1);
        }

        let curr_row = self.content.char_to_line(index);
        let curr_row_start = self.content.line_to_char(curr_row);
        let curr_col = index - curr_row_start;

        (curr_col, curr_row)
    }

    pub fn find_char_after_separator(&self, cursor: &Cursor) -> (usize, usize) {
        let start_idx = self.content.line_to_char(cursor.row()).add(cursor.col());
        let mut end_idx = 0;
        let mut found_newline = false;

        if let Some(initial_char) = self.content.get_char(start_idx) {
            for char in self.content.chars_at(start_idx) {
                match (
                    initial_char.is_alphanumeric(),
                    char.is_alphanumeric(),
                    found_newline,
                ) {
                    (_, _, true) if !char.is_whitespace() => break,
                    (false, true, _) => break,
                    (true, false, _) => break,
                    _ if char.is_whitespace() => {
                        found_newline = true;
                        end_idx = end_idx.add(1);
                    }
                    _ => end_idx = end_idx.add(1),
                }
            }
        }

        let curr_idx = start_idx.add(end_idx);
        let curr_row = self.content.char_to_line(curr_idx);
        let curr_row_start = self.content.line_to_char(curr_row);
        let curr_col = curr_idx.sub(curr_row_start);

        (curr_col, curr_row)
    }

    pub fn find_char_before_separator(&self, cursor: &Cursor) -> (usize, usize) {
        let start_idx = self.content.line_to_char(cursor.row()).add(cursor.col());
        let mut end_idx = start_idx;
        let mut found_newline = false;

        if let Some(initial_char) = self.content.get_char(start_idx) {
            for _ in (0..start_idx.saturating_sub(1)).rev() {
                let char = self.content.char(end_idx);

                match (
                    initial_char.is_alphanumeric(),
                    char.is_alphanumeric(),
                    found_newline,
                ) {
                    (_, _, true) if !self.line_break.to_string().contains(char) => break,
                    (false, true, _) => break,
                    (true, false, _) => break,
                    _ if self.line_break.to_string().contains(char) => {
                        found_newline = true;
                        end_idx = end_idx.saturating_sub(1);
                    }
                    _ => end_idx = end_idx.saturating_sub(1),
                }
            }
        };

        let curr_row = self.content.char_to_line(end_idx);
        let curr_row_start = self.content.line_to_char(curr_row);
        let curr_col = end_idx.sub(curr_row_start);

        (curr_col, curr_row)
    }

    pub fn find_empty_line_above(&self, cursor: &Cursor) -> usize {
        let mut new_row = cursor.row().saturating_sub(1);

        while let Some(line) = self.content.get_line(new_row) {
            if line.to_string().eq(&self.line_break.to_string()) {
                break;
            }

            if new_row.eq(&0) {
                break;
            }
            new_row = new_row.saturating_sub(1);
        }

        new_row
    }

    pub fn find_empty_line_below(&self, cursor: &Cursor) -> usize {
        let mut new_row = cursor.row().add(1);
        let len_lines = self.len_lines();

        while let Some(line) = self.content.get_line(new_row) {
            if line.to_string().eq(&self.line_break.to_string()) {
                break;
            }
            new_row = new_row.add(1);
        }

        usize::min(new_row, len_lines.saturating_sub(1))
    }

    pub fn len_lines(&self) -> usize {
        self.content.len_lines()
    }

    pub fn delete_line(&mut self, line: usize) {
        let start = self.content.line_to_char(line);
        let end = self.content.line_to_char(line.add(1));
        self.content.try_remove(start..end).ok();
    }

    /// deletes a word forward in one of two ways:
    ///
    /// - if the current character is alphanumeric, then this delete up to the first non
    /// alphanumeric character
    /// - if the current character is non alphanumeric, then delete up to the first alphanumeric
    /// character
    pub fn delete_word(&mut self, cursor: &Cursor) {
        let start_idx = self.content.line_to_char(cursor.row()).add(cursor.col());
        let mut end_idx = start_idx.saturating_sub(1);

        if let Some(initial_char) = self.content.get_char(start_idx) {
            for char in self.content.chars_at(start_idx) {
                match (initial_char.is_alphanumeric(), char.is_alphanumeric()) {
                    (false, _) if self.line_break.to_string().contains(char) => break,
                    (false, true) => {
                        end_idx = end_idx.add(1);
                        break;
                    }
                    (true, false) => {
                        end_idx = end_idx.add(1);
                        break;
                    }
                    _ => end_idx = end_idx.add(1),
                }
            }

            self.content.try_remove(start_idx..end_idx).ok();
        }
    }

    /// deletes a word backwards in one of two ways:
    ///
    /// - if the current character is alphanumeric, then this delete up to the first non
    /// alphanumeric character
    /// - if the current character is non alphanumeric, then delete up to the first alphanumeric
    /// character
    ///
    /// will always return how many columns to advance the cursor
    pub fn delete_word_backwards(&mut self, cursor: &Cursor) -> usize {
        let start_idx = self.content.line_to_char(cursor.row()).add(cursor.col());
        let mut end_idx = start_idx.saturating_sub(1);

        if let Some(initial_char) = self.content.get_char(start_idx) {
            for _ in (0..start_idx.saturating_sub(1)).rev() {
                let char = self.content.char(end_idx);
                match (initial_char.is_alphanumeric(), char.is_alphanumeric()) {
                    (false, _) if self.line_break.to_string().contains(char) => break,
                    (false, true) => break,
                    (true, false) => break,
                    _ => end_idx = end_idx.saturating_sub(1),
                }
            }
        };

        end_idx.sub(start_idx)
    }

    pub fn insert_line_below(&mut self, cursor: &Cursor, tree: Option<&Tree>) {
        let indentation = if let Some(tree) = tree {
            let line_byte_idx = self.content.line_to_byte(cursor.row());
            let cursor_byte_idx = line_byte_idx.add(cursor.col());
            let indentation_level = Highlighter::find_indentation_level(tree, cursor_byte_idx);
            tracing::debug!("{indentation_level}");
            "  ".repeat(indentation_level)
        } else {
            String::new()
        };
        let next_line = self.content.line_to_char(cursor.row().add(1));
        let line_with_indentation = format!("{}{}", indentation, &self.line_break.to_string());
        self.content.insert(next_line, &line_with_indentation);
    }

    pub fn insert_line_above(&mut self, cursor: &Cursor) {
        let curr_line = self.content.line_to_char(cursor.row());
        self.content.insert(curr_line, &self.line_break.to_string());
    }

    pub fn find_oposing_token(&mut self, cursor: &Cursor) -> (usize, usize) {
        let start_idx = self.content.line_to_char(cursor.row()).add(cursor.col());
        let mut combinations = HashMap::new();
        let pairs = [('<', '>'), ('(', ')'), ('[', ']'), ('{', '}')];
        pairs.iter().for_each(|pair| {
            combinations.insert(pair.0, pair.1);
            combinations.insert(pair.1, pair.0);
        });

        let mut look_forward = true;
        let mut token_to_search = char::default();
        let (mut curr_open, mut walked) = (0, 0);

        if let Some(initial_char) = self.content.get_char(start_idx) {
            match initial_char {
                c if is_opening_token(c) => {
                    token_to_search = *combinations.get(&c).unwrap();
                    curr_open = curr_open.add(1);
                }
                c if is_closing_token(c) => {
                    token_to_search = *combinations.get(&c).unwrap();
                    curr_open = curr_open.add(1);
                    look_forward = false;
                }
                _ => {}
            }

            let range = if look_forward {
                start_idx.add(1)..self.content.len_chars()
            } else {
                0..start_idx
            };

            for i in range {
                let char = self
                    .content
                    .get_char(if look_forward {
                        i
                    } else {
                        start_idx - walked - 1
                    })
                    .unwrap_or_default();

                if token_to_search.eq(&char::default()) {
                    if !is_opening_token(char) {
                        walked = walked.add(1);
                        continue;
                    }
                    token_to_search = *combinations.get(&char).unwrap();
                }

                char.eq(combinations.get(&token_to_search).unwrap())
                    .then(|| curr_open = curr_open.add(1));

                char.eq(&token_to_search)
                    .then(|| curr_open = curr_open.sub(1));

                walked = walked.add(1);

                if curr_open.eq(&0) {
                    break;
                }
            }
        }

        if curr_open.gt(&0) {
            return (cursor.col(), cursor.row());
        }

        if look_forward {
            let curr_row = self.content.char_to_line(start_idx.add(walked));
            let curr_row_start = self.content.line_to_char(curr_row);
            let curr_col = start_idx.add(walked).saturating_sub(curr_row_start);
            (curr_col, curr_row)
        } else {
            let curr_row = self.content.char_to_line(start_idx.sub(walked));
            let curr_row_start = self.content.line_to_char(curr_row);
            let curr_col = start_idx.sub(walked).sub(curr_row_start);
            (curr_col, curr_row)
        }
    }
}

impl<State> std::fmt::Display for TextObject<State> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.content.to_string())
    }
}

fn is_opening_token(char: char) -> bool {
    matches!(char, '(' | '{' | '[' | '<')
}

fn is_closing_token(char: char) -> bool {
    matches!(char, ')' | '}' | ']' | '>')
}
