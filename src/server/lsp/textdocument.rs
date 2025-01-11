use std::{
    fmt::{self, Display},
    usize,
};

use log::error;
use serde::{Deserialize, Serialize};

use tree_sitter::{Node, Point};

use super::TextDocumentContentChangeEvent;

pub type DocumentUri = String;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TextDocumentItem {
    pub uri: DocumentUri,
    language_id: String,
    version: u32,
    pub text: String,
}

impl TextDocumentItem {
    #[cfg(test)]
    pub(crate) fn new(uri: &str, text: &str) -> TextDocumentItem {
        TextDocumentItem {
            uri: uri.to_string(),
            text: text.to_string(),
            language_id: "sparql".to_string(),
            version: 0,
        }
    }

    fn apply_text_edit(&mut self, text_edit: TextEdit) {
        match text_edit.range.to_byte_index_range(&self.text) {
            Some(range) => {
                self.text.replace_range(range, &text_edit.new_text);
            }
            None => {
                error!("Received textdocument/didChange notification with a TextEdit thats out ouf bounds:\nedit: {}\ndocument range: {}",text_edit, self.get_full_range());
            }
        };

        // WARNING: Always keep one newline at the end of a document to stay POSIX conform!
        // https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/V1_chap03.html#tag_03_206
        match self.text.chars().rev().next() {
            Some('\n') => {}
            _ => self.text.push('\n'),
        };
    }

    pub(crate) fn apply_text_edits(&mut self, text_edits: Vec<TextEdit>) {
        for text_edit in text_edits {
            self.apply_text_edit(text_edit);
        }
    }

    pub fn get_full_range(&self) -> Range {
        if self.text.is_empty() {
            return Range::new(0, 0, 0, 0);
        }
        let line_count = self.text.lines().count();
        let last_char = self
            .text
            .chars()
            .rev()
            .next()
            .expect("At least one character has to be in the text");
        match last_char {
            '\n' => Range::new(0, 0, line_count as u32, 0),
            _ => {
                let last_line = self
                    .text
                    .lines()
                    .rev()
                    .next()
                    .expect("At least one line hat to be in the text");
                Range::new(0, 0, (line_count - 1) as u32, last_line.len() as u32)
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct VersionedTextDocumentIdentifier {
    #[serde(flatten)]
    pub base: TextDocumentIdentifier,
    version: u32,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct TextDocumentIdentifier {
    pub uri: Uri,
}

type Uri = String;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
pub struct Position {
    line: u32,
    character: u32,
}

impl Position {
    pub fn new(line: u32, character: u32) -> Self {
        Self { line, character }
    }

    pub(crate) fn to_point(&self) -> Point {
        Point {
            row: self.line as usize,
            column: self.character as usize,
        }
    }

    pub fn to_byte_index(&self, text: &String) -> Option<usize> {
        if self.line == 0 && self.character == 0 && text.is_empty() {
            return Some(0);
        }
        let mut byte_index: usize = 0;
        let mut lines = text.lines();
        for _i in 0..self.line {
            byte_index += lines.next()?.len() + 1;
        }

        std::iter::once(byte_index)
            .chain(
                lines
                    .next()
                    .unwrap_or("")
                    .chars()
                    .scan(byte_index, |accu, c| {
                        *accu += c.len_utf8();
                        Some(*accu)
                    }),
            )
            .nth(self.character as usize)
    }
}

impl PartialOrd for Position {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Position {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.line.cmp(&other.line) {
            std::cmp::Ordering::Equal => self.character.cmp(&other.character),
            x => x,
        }
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.character)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#range
// NOTE: Positions are zero based.
// NOTE: The end position is exclusive.
// NOTE: To include line ending character(s), set end position to the start of next line.
pub struct Range {
    pub start: Position,
    pub end: Position,
}

impl Range {
    pub fn new(start_line: u32, start_character: u32, end_line: u32, end_character: u32) -> Self {
        Self {
            start: Position::new(start_line, start_character),
            end: Position::new(end_line, end_character),
        }
    }

    pub(crate) fn from_node(node: Node) -> Range {
        Self {
            start: Position::new(
                node.start_position().row as u32,
                node.start_position().column as u32,
            ),
            end: Position::new(
                node.end_position().row as u32,
                node.end_position().column as u32,
            ),
        }
    }

    pub fn to_byte_index_range(&self, text: &String) -> Option<std::ops::Range<usize>> {
        match (self.start.to_byte_index(text), self.end.to_byte_index(text)) {
            (Some(from), Some(to)) => Some(from..to),
            _ => None,
        }
    }
}

impl Display for Range {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&format!("{}-{}", self.start, self.end))
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TextEdit {
    range: Range,
    new_text: String,
}

impl TextEdit {
    pub fn new(range: Range, new_text: &str) -> Self {
        Self {
            range,
            new_text: new_text.to_string(),
        }
    }

    pub fn from_text_document_content_change_event(
        change_event: TextDocumentContentChangeEvent,
    ) -> Self {
        // TODO: handle option: change events has no range (whole document got send)
        Self {
            range: change_event.range,
            new_text: change_event.text,
        }
    }
}

impl Display for TextEdit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&format!("{} \"{}\"", self.range, self.new_text))
    }
}

#[cfg(test)]
mod tests {

    use indoc::indoc;

    use crate::server::lsp::textdocument::{Position, Range, TextEdit};

    use super::TextDocumentItem;

    #[test]
    fn full_range_empty() {
        let document: TextDocumentItem = TextDocumentItem {
            uri: "file:///dings".to_string(),
            language_id: "foo".to_string(),
            version: 1,
            text: "".to_string(),
        };
        assert_eq!(document.get_full_range(), Range::new(0, 0, 0, 0));
    }

    #[test]
    fn full_range_trailing_newline() {
        let document: TextDocumentItem = TextDocumentItem {
            uri: "file:///dings".to_string(),
            language_id: "foo".to_string(),
            version: 1,
            text: "abc\nde\n".to_string(),
        };
        assert_eq!(document.get_full_range(), Range::new(0, 0, 2, 0));
    }

    #[test]
    fn full_range_no_trailing_newline() {
        let document: TextDocumentItem = TextDocumentItem {
            uri: "file:///dings".to_string(),
            language_id: "foo".to_string(),
            version: 1,
            text: "abc\nde".to_string(),
        };
        assert_eq!(document.get_full_range(), Range::new(0, 0, 1, 2));
    }

    #[test]
    fn changes() {
        let mut document: TextDocumentItem = TextDocumentItem {
            uri: "file:///dings".to_string(),
            language_id: "foo".to_string(),
            version: 1,
            text: "".to_string(),
        };
        assert_eq!(document.text, "");
        document.apply_text_edit(TextEdit {
            new_text: "S".to_string(),
            range: Range::new(0, 0, 0, 0),
        });
        assert_eq!(document.text, "S\n");
        document.apply_text_edits(vec![
            TextEdit {
                new_text: "E".to_string(),
                range: Range::new(0, 1, 0, 1),
            },
            TextEdit {
                new_text: "L".to_string(),
                range: Range::new(0, 2, 0, 2),
            },
            TextEdit {
                new_text: "E".to_string(),
                range: Range::new(0, 3, 0, 3),
            },
            TextEdit {
                new_text: "C".to_string(),
                range: Range::new(0, 4, 0, 4),
            },
            TextEdit {
                new_text: "T".to_string(),
                range: Range::new(0, 5, 0, 5),
            },
            TextEdit {
                new_text: " ".to_string(),
                range: Range::new(0, 6, 0, 6),
            },
            TextEdit {
                new_text: "* WHERE{\n  ?s ?p ?o\n}".to_string(),
                range: Range::new(0, 7, 0, 7),
            },
        ]);
        assert_eq!(document.text, "SELECT * WHERE{\n  ?s ?p ?o\n}\n");
        document.apply_text_edits(vec![TextEdit {
            new_text: "select".to_string(),
            range: Range::new(0, 0, 0, 6),
        }]);
        assert_eq!(document.text, "select * WHERE{\n  ?s ?p ?o\n}\n");
        document.apply_text_edits(vec![
            TextEdit {
                new_text: "".to_string(),
                range: Range::new(1, 10, 2, 0),
            },
            TextEdit {
                new_text: "".to_string(),
                range: Range::new(0, 15, 1, 1),
            },
        ]);
        assert_eq!(document.text, "select * WHERE{ ?s ?p ?o}\n");
        document.apply_text_edits(vec![
            TextEdit {
                new_text: "ns1:dings".to_string(),
                range: Range::new(0, 16, 0, 18),
            },
            TextEdit {
                new_text: "PREFIX ns1: <iri>\n".to_string(),
                range: Range::new(0, 0, 0, 0),
            },
        ]);
        assert_eq!(
            document.text,
            "PREFIX ns1: <iri>\nselect * WHERE{ ns1:dings ?p ?o}\n"
        );
        document.apply_text_edits(vec![
            TextEdit {
                new_text: "".to_string(),
                range: Range::new(1, 10, 1, 32),
            },
            TextEdit {
                new_text: "".to_string(),
                range: Range::new(0, 0, 1, 10),
            },
        ]);
        // Whats goning on here
        assert_eq!(document.text, "\n");
    }

    #[test]
    fn apply_change() {
        let mut document: TextDocumentItem = TextDocumentItem {
            uri: "file:///dings".to_string(),
            language_id: "foo".to_string(),
            version: 1,
            text: "\n".to_string(),
        };
        let change = TextEdit {
            new_text: "dings".to_string(),
            range: Range::new(0, 0, 0, 0),
        };
        document.apply_text_edit(change);
        assert_eq!(document.text, "dings\n");
    }

    #[test]
    fn position_to_byte_index() {
        let text = "aä�𐀀".to_string();
        assert_eq!(Position::new(0, 0).to_byte_index(&text), Some(0));
        assert_eq!(Position::new(0, 1).to_byte_index(&text), Some(1));
        assert_eq!(Position::new(0, 2).to_byte_index(&text), Some(3));
        assert_eq!(Position::new(0, 3).to_byte_index(&text), Some(6));
        assert_eq!(Position::new(0, 4).to_byte_index(&text), Some(10));
        assert_eq!(Position::new(1, 0).to_byte_index(&text), Some(11));
        assert_eq!(Position::new(0, 5).to_byte_index(&text), None);
        assert_eq!(Position::new(2, 0).to_byte_index(&text), None);
    }

    #[test]
    fn range_to_byte_index_range() {
        let text = indoc!(
            "12345
             12345
             12345
             "
        )
        .to_string();
        assert_eq!(
            Range::new(0, 5, 1, 1).to_byte_index_range(&text),
            Some(5..7)
        );
        let range = Range::new(1, 0, 2, 0);
        let pos = range.start;
        assert_eq!(pos.to_byte_index(&text), Some(6));
        assert_eq!(
            Range::new(1, 0, 2, 0).to_byte_index_range(&text),
            Some(6..12)
        );
        assert_eq!(
            Range::new(0, 0, 3, 0).to_byte_index_range(&text),
            Some(0..18)
        );

        assert_eq!(Range::new(0, 0, 3, 1).to_byte_index_range(&text), None);
        assert_eq!(Range::new(0, 0, 1, 10).to_byte_index_range(&text), None);
    }

    #[test]
    fn no_changes() {
        let changes: Vec<TextEdit> = vec![];
        let mut document: TextDocumentItem = TextDocumentItem {
            uri: "file:///dings".to_string(),
            language_id: "foo".to_string(),
            version: 1,
            text: "hello world\n".to_string(),
        };
        document.apply_text_edits(changes);
        assert_eq!(document.text, "hello world\n");
    }
}
