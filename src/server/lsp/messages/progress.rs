use serde::{Deserialize, Serialize};

use crate::server::lsp::{
    rpc::NotificationMessageBase,
    workdoneprogress::{
        ProgressToken, ProgressValue, WorkDoneProgressBegin, WorkDoneProgressEnd,
        WorkDoneProgressReport,
    },
};

#[derive(Debug, Serialize, Deserialize)]
pub struct ProgressNotification {
    #[serde(flatten)]
    pub base: NotificationMessageBase,
    pub params: ProgressParams<ProgressValue>,
}

impl ProgressNotification {
    pub fn begin_notification(
        token: ProgressToken,
        title: &str,
        cancellable: Option<bool>,
        message: Option<&str>,
        percentage: Option<u32>,
    ) -> Self {
        ProgressNotification {
            base: NotificationMessageBase::new("$/progress"),
            params: ProgressParams {
                token,
                value: ProgressValue::Begin(WorkDoneProgressBegin::new(
                    title,
                    cancellable,
                    message,
                    percentage,
                )),
            },
        }
    }

    pub(crate) fn report_notification(
        token: ProgressToken,
        cancellable: Option<bool>,
        message: Option<&str>,
        percentage: Option<u32>,
    ) -> Self {
        ProgressNotification {
            base: NotificationMessageBase::new("$/progress"),
            params: ProgressParams {
                token,
                value: ProgressValue::Report(WorkDoneProgressReport::new(
                    cancellable,
                    message,
                    percentage,
                )),
            },
        }
    }

    pub(crate) fn end_notification(token: ProgressToken, message: Option<&str>) -> Self {
        ProgressNotification {
            base: NotificationMessageBase::new("$/progress"),
            params: ProgressParams {
                token,
                value: ProgressValue::End(WorkDoneProgressEnd::new(message)),
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProgressParams<T> {
    token: ProgressToken,
    value: T,
}

#[cfg(test)]
mod tests {
    use crate::server::lsp::{
        rpc::NotificationMessageBase,
        workdoneprogress::{
            ProgressValue, WorkDoneProgressBegin, WorkDoneProgressEnd, WorkDoneProgressReport,
        },
        ProgressNotification, ProgressParams,
    };

    use super::ProgressToken;

    #[test]
    fn serialize() {
        let progress_begin = ProgressNotification {
            base: NotificationMessageBase::new("$/progress"),
            params: ProgressParams {
                token: ProgressToken::Integer(1),
                value: ProgressValue::Begin(WorkDoneProgressBegin::new(
                    "progress title",
                    Some(false),
                    Some("progress message"),
                    Some(0),
                )),
            },
        };
        assert_eq!(
            serde_json::to_string(&progress_begin).unwrap(),
            r#"{"jsonrpc":"2.0","method":"$/progress","params":{"token":1,"value":{"kind":"begin","title":"progress title","cancellable":false,"message":"progress message","percentage":0}}}"#
        );
        let progress_report = ProgressNotification {
            base: NotificationMessageBase::new("$/progress"),
            params: ProgressParams {
                token: ProgressToken::Text("1337-42".to_string()),
                value: ProgressValue::Report(WorkDoneProgressReport::new(
                    Some(false),
                    Some("progress message"),
                    Some(50),
                )),
            },
        };
        assert_eq!(
            serde_json::to_string(&progress_report).unwrap(),
            r#"{"jsonrpc":"2.0","method":"$/progress","params":{"token":"1337-42","value":{"kind":"report","cancellable":false,"message":"progress message","percentage":50}}}"#
        );
        let progress_end = ProgressNotification {
            base: NotificationMessageBase::new("$/progress"),
            params: ProgressParams {
                token: ProgressToken::Text("1337-42".to_string()),
                value: ProgressValue::End(WorkDoneProgressEnd::new(Some("progress message"))),
            },
        };
        assert_eq!(
            serde_json::to_string(&progress_end).unwrap(),
            r#"{"jsonrpc":"2.0","method":"$/progress","params":{"token":"1337-42","value":{"kind":"end","message":"progress message"}}}"#
        );
    }
}
