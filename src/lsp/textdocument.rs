use log::error;
use serde::{Deserialize, Serialize};

use super::TextDocumentContentChangeEvent;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TextDoucmentItem {
    pub uri: String,
    language_id: String,
    version: u32,
    pub text: String,
}

impl TextDoucmentItem {
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

#[cfg(test)]
mod tests {
    use crate::lsp::TextDocumentContentChangeEvent;

    use super::TextDoucmentItem;

    #[test]
    fn full_changes() {
        let changes: Vec<TextDocumentContentChangeEvent> = vec![TextDocumentContentChangeEvent {
            text: "goodbye world".to_string(),
        }];
        let mut document: TextDoucmentItem = TextDoucmentItem {
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
        let mut document: TextDoucmentItem = TextDoucmentItem {
            uri: "file:///dings".to_string(),
            language_id: "foo".to_string(),
            version: 1,
            text: "hello world".to_string(),
        };
        document.apply_changes(changes);
        assert_eq!(document.text, "hello world");
    }
}
