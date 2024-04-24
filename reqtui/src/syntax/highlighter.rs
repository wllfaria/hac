use tree_sitter::{Parser, Query, QueryCursor};

pub struct Highlighter {
    parser: Parser,
    query: Query,
}

#[derive(Debug)]
pub struct ColorInfo {
    pub start: usize,
    pub end: usize,
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
    pub fn apply(&mut self, buffer: &str) -> Vec<ColorInfo> {
        let tree = self.parser.parse(buffer, None).unwrap();

        let mut colors = Vec::new();
        let mut cursor = QueryCursor::new();
        let matches = cursor.matches(&self.query, tree.root_node(), buffer.as_bytes());

        for m in matches {
            for cap in m.captures {
                let node = cap.node;
                let start = node.start_byte();
                let end = node.end_byte();
                let _capture_name = self.query.capture_names()[cap.index as usize];
                colors.push(ColorInfo { start, end });
            }
        }

        colors
    }
}
