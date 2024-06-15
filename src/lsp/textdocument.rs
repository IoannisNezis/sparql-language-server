use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TextDoucmentItem {
    pub uri: String,
    language_id: String,
    version: u32,
    pub text: String,
}
