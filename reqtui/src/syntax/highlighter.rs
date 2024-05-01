use lazy_static::lazy_static;
use ratatui::style::Style;

use std::{collections::HashMap, fmt::Debug, sync::RwLock};

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
    pub fn parse<'a>(&mut self, buffer: &str) -> Option<Tree> {
        self.parser.parse(buffer, None)
    }

    pub fn apply(
        &self,
        buffer: &str,
        tree: Option<&Tree>,
        tokens: &HashMap<String, Style>,
    ) -> Vec<ColorInfo> {
        let mut colors = Vec::new();

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
                        colors.push(ColorInfo {
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
}
