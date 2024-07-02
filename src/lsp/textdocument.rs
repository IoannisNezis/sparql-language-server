use std::fmt;

use log::error;
use serde::{Deserialize, Serialize};
use tree_sitter::{Node, Point};

use super::TextDocumentContentChangeEvent;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TextDocumentItem {
    pub uri: String,
    language_id: String,
    version: u32,
    pub text: String,
}

impl TextDocumentItem {
    pub(crate) fn apply_changes(
        &mut self,
        mut content_canges: Vec<TextDocumentContentChangeEvent>,
    ) {
        match content_canges.first_mut() {
            Some(change) => self.text = std::mem::take(&mut change.text),
            None => {
                error!(
                    "revieved empty vector of changes for document: {}",
                    self.uri
                );
            }
        };
    }

    pub(crate) fn extract_node(&self, node: Node) -> Option<&str> {
        self.text.get(node.start_byte()..node.end_byte())
    }

    pub fn get_full_range(&self) -> Range {
        let lines = self.text.lines().collect::<Vec<&str>>();
        return Range::new(
            0,
            0,
            lines.len() as u32,
            lines.last().unwrap_or(&"").len() as u32,
        );
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

#[derive(Debug, Serialize, Deserialize, PartialEq)]
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

    fn from_point(point: Point) -> Position {
        Position {
            line: point.row as u32,
            character: point.column as u32,
        }
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.character)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Range {
    start: Position,
    end: Position,
}

impl Range {
    pub fn from_tsrange(range: tree_sitter::Range) -> Self {
        Self {
            start: Position::from_point(range.start_point),
            end: Position::from_point(range.end_point),
        }
    }

    fn new(start_line: u32, start_character: u32, end_line: u32, end_character: u32) -> Self {
        Self {
            start: Position::new(start_line, start_character),
            end: Position::new(end_line, end_character),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TextEdit {
    range: Range,
    new_text: String,
}

impl TextEdit {
    pub fn new(range: Range, new_text: String) -> Self {
        Self { range, new_text }
    }

    pub(crate) fn foo() -> Vec<TextEdit> {
        vec![TextEdit {
            range: Range {
                start: Position::new(0, 0),
                end: Position::new(0, 1),
            },
            new_text: "hello from lsp".to_string(),
        }]
    }
}

#[cfg(test)]
mod tests {
    use crate::lsp::TextDocumentContentChangeEvent;

    use super::TextDocumentItem;

    #[test]
    fn full_changes() {
        let changes: Vec<TextDocumentContentChangeEvent> = vec![TextDocumentContentChangeEvent {
            text: "goodbye world".to_string(),
        }];
        let mut document: TextDocumentItem = TextDocumentItem {
            uri: "file:///dings".to_string(),
            language_id: "foo".to_string(),
            version: 1,
            text: "hello world".to_string(),
        };

        document.apply_changes(changes);
        assert_eq!(document.text, "goodbye world");
    }

    #[test]
    fn no_changes() {
        let changes: Vec<TextDocumentContentChangeEvent> = vec![];
        let mut document: TextDocumentItem = TextDocumentItem {
            uri: "file:///dings".to_string(),
            language_id: "foo".to_string(),
            version: 1,
            text: "hello world".to_string(),
        };
        document.apply_changes(changes);
        assert_eq!(document.text, "hello world");
    }
}
