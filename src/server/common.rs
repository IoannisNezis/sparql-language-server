use std::{any::type_name, fmt};

use log::error;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use super::lsp::{
    errors::{ErrorCode, ResponseError},
    textdocument::DocumentUri,
};

pub(crate) fn serde_parse<T, O>(message: O) -> Result<T, ResponseError>
where
    T: Serialize + DeserializeOwned,
    O: Serialize + fmt::Debug,
{
    match serde_json::to_string(&message) {
        Ok(serialized_message) => serde_json::from_str(&serialized_message).map_err(|error| {
            error!(
                "Error while deserializing message:\n{}-----------------------\n{:?}",
                error, message,
            );
            ResponseError::new(
                ErrorCode::ParseError,
                &format!(
                    "Could not deserialize RPC-message \"{}\"\n\n{}",
                    type_name::<T>(),
                    error
                ),
            )
        }),
        Err(error) => Err(ResponseError::new(
            ErrorCode::ParseError,
            &format!("Could not serialize RPC-message\n\n{}", error),
        )),
    }
}

/// This struct represents diagnostic data from the uncompacted-uri diagnostic.
///
/// The fields are:
/// - `prefix`: The prefix associated with the namespace.
/// - `namespace`: The namespace URI.
/// - `curie`: The compact URI (CURIE).
#[derive(Debug, Serialize, Deserialize)]
pub struct UncompactedUrisDiagnosticData(pub String, pub String, pub String);

/// This struct represents diagnostic data from the uncompacted-uri diagnostic.
///
/// The fields are:
/// - `prefix`: The prefix associated with the namespace.
/// - `namespace`: The namespace URI.
/// - `curie`: The compact URI (CURIE).
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
