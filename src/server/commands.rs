use serde::{Deserialize, Serialize};

use super::lsp::textdocument::DocumentUri;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct PublishDiagnosticsCommandAruments(pub (DocumentUri,));

#[cfg(test)]
mod test {
    use super::PublishDiagnosticsCommandAruments;

    #[test]
    fn serialize() {
        let arguments = PublishDiagnosticsCommandAruments(("file.rq".to_string(),));
        let serialized_arguments = serde_json::to_string(&arguments).unwrap();
        assert_eq!(serialized_arguments, r#"["file.rq"]"#);
    }

    #[test]
    fn deserialize() {
        let serialized_arguments = r#"["file.rq"]"#;
        let arguments = PublishDiagnosticsCommandAruments(("file.rq".to_string(),));
        let deserialized_args: PublishDiagnosticsCommandAruments =
            serde_json::from_str(serialized_arguments).unwrap();
        assert_eq!(deserialized_args, arguments);
    }
}
