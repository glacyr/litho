use litho_language::lex::Span;
use tower_lsp::lsp_types::{Position, Range};

pub fn line_col_to_offset(source: &str, line: u32, col: u32) -> usize {
    let line_offset = source
        .split_inclusive("\n")
        .take(line as usize)
        .fold(0, |sum, line| sum + line.len());
    line_offset + col as usize
}

pub fn index_to_position(source: &str, index: usize) -> Position {
    let mut line = 0;
    let mut character = 0;

    for char in source[0..index].chars() {
        if char == '\n' {
            line += 1;
            character = 0;
        } else {
            character += 1;
        }
    }

    Position { line, character }
}

pub fn span_to_range(source: &str, span: Span) -> Range {
    Range {
        start: index_to_position(source, span.start),
        end: index_to_position(source, span.end),
    }
}
