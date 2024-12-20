use serde::{Deserialize, Serialize};

use crate::server::lsp::rpc::NotificationMessageBase;

#[derive(Debug, Serialize, Deserialize)]
pub struct SetTraceNotification {
    #[serde(flatten)]
    pub base: NotificationMessageBase,
    pub params: SetTraceParams,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetTraceParams {
    pub value: TraceValue,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TraceValue {
    Off,
    Message,
    Verbose,
}

#[cfg(test)]
mod test {

    use crate::server::lsp::rpc::NotificationMessageBase;

    use super::{SetTraceNotification, TraceValue};

    #[test]
    fn serialize() {
        let set_trace_notification = SetTraceNotification {
            base: NotificationMessageBase::new("$/setTrace"),
            params: super::SetTraceParams {
                value: TraceValue::Off,
            },
        };
        assert_eq!(
            serde_json::to_string(&set_trace_notification).unwrap(),
            r#"{"jsonrpc":"2.0","method":"$/setTrace","params":{"value":"off"}}"#
        )
    }
}
