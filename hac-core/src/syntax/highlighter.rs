use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;
use std::sync::RwLock;

use lazy_static::lazy_static;
use ratatui::style::Style;
use tree_sitter::{Parser, Query, QueryCursor, Tree};

lazy_static! {
    pub static ref HIGHLIGHTER: RwLock<Highlighter> = RwLock::new(Highlighter::default());
}

pub struct Highlighter {
    parser: Parser,
    query: Query,
}

impl Debug for Highlighter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Highlighter").finish()
    }
}

#[derive(Debug, PartialEq)]
pub struct ColorInfo {
    pub start: usize,
    pub end: usize,
    pub style: Style,
}

impl Default for Highlighter {
    fn default() -> Self {
        let mut parser = Parser::new();
        let json_language = include_str!("queries/json/highlights.scm");
        let query = Query::new(&tree_sitter_json::language(), json_language)
            .expect("failed to load json query");

        parser
            .set_language(&tree_sitter_json::language())
            .expect("error loading json grammar");

        Highlighter { parser, query }
    }
}

impl Highlighter {
    pub fn parse(&mut self, buffer: &str) -> Option<Tree> {
        self.parser.parse(buffer, None)
    }

    pub fn apply(
        &self,
        buffer: &str,
        tree: Option<&Tree>,
        tokens: &HashMap<String, Style>,
    ) -> VecDeque<ColorInfo> {
        let mut colors = VecDeque::new();

        if let Some(tree) = tree {
            let mut cursor = QueryCursor::new();
            let matches = cursor.matches(&self.query, tree.root_node(), buffer.as_bytes());

            for m in matches {
                for cap in m.captures {
                    let node = cap.node;
                    let start = node.start_byte();
                    let end = node.end_byte();
                    let capture_name = self.query.capture_names()[cap.index as usize];
                    if let Some(style) = tokens.get(capture_name) {
                        colors.push_back(ColorInfo {
                            start,
                            end,
                            style: *style,
                        });
                    }
                }
            }
        }

        colors
    }

    pub fn find_indentation_level(tree: &Tree, cursor_byte_idx: usize) -> usize {
        let root_node = tree.root_node();
        let current_node = root_node
            .descendant_for_byte_range(cursor_byte_idx, cursor_byte_idx)
            .unwrap();
        let mut indent_level: usize = 0;
        let mut current_node = current_node;
        while let Some(parent) = current_node.parent() {
            if parent.kind().eq("pair") {
                current_node = parent;
                continue;
            }
            current_node = parent;
            indent_level += 1;
        }
        indent_level.saturating_sub(1)
    }
}
