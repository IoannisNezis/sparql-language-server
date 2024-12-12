use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(untagged)]
pub enum ProgressToken {
    Integer(i32),
    Text(String),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
// NOTE: Docs: https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#workDoneProgress
pub enum ProgressValue {
    Begin(WorkDoneProgressBegin),
    Report(WorkDoneProgressReport),
    End(WorkDoneProgressEnd),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkDoneProgressBegin {
    kind: String,
    title: String,
    cancellable: Option<bool>,
    message: Option<String>,
    percentage: Option<u32>,
}

impl WorkDoneProgressBegin {
    pub fn new(
        title: &str,
        cancellable: Option<bool>,
        message: Option<&str>,
        percentage: Option<u32>,
    ) -> Self {
        Self {
            kind: "begin".to_string(),
            title: title.to_string(),
            cancellable,
            message: message.map(|str| str.to_string()),
            percentage,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkDoneProgressReport {
    kind: String,
    cancellable: Option<bool>,
    message: Option<String>,
    percentage: Option<u32>,
}

impl WorkDoneProgressReport {
    pub fn new(cancellable: Option<bool>, message: Option<&str>, percentage: Option<u32>) -> Self {
        Self {
            kind: "report".to_string(),
            cancellable,
            message: message.map(|str| str.to_string()),
            percentage,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkDoneProgressEnd {
    kind: String,
    message: Option<String>,
}

impl WorkDoneProgressEnd {
    pub fn new(message: Option<&str>) -> Self {
        Self {
            kind: "end".to_string(),
            message: message.map(|str| str.to_string()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WorkDoneProgressParams {
    pub work_done_token: Option<ProgressToken>,
}
