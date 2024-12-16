use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum LSPAny {
    LSPObject(HashMap<String, LSPAny>),
    LSPArray(Vec<LSPAny>),
    String(String),
    Uinteger(u32),
    Integer(i32),
    Decimal(f32),
    Boolean(bool),
    Null,
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use super::LSPAny;
    fn serialize_and_compare(data: LSPAny, expected_result: &str) {
        let result = serde_json::to_string(&data).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn serialize() {
        let lsp_object = LSPAny::LSPObject(HashMap::from([("a".to_string(), LSPAny::Integer(42))]));
        serialize_and_compare(lsp_object, r#"{"a":42}"#);
        let lsp_array =
            LSPAny::LSPArray(vec![LSPAny::String("hay".to_string()), LSPAny::Integer(1)]);
        serialize_and_compare(lsp_array, r#"["hay",1]"#);
        let string = LSPAny::String("hay".to_string());
        serialize_and_compare(string, r#""hay""#);
        let integer = LSPAny::Integer(-42);
        serialize_and_compare(integer, r#"-42"#);
        let uinteger = LSPAny::Uinteger(42);
        serialize_and_compare(uinteger, r#"42"#);
        let decimal = LSPAny::Decimal(1.1);
        serialize_and_compare(decimal, r#"1.1"#);
        let boolean = LSPAny::Boolean(false);
        serialize_and_compare(boolean, r#"false"#);
        let null = LSPAny::Null;
        serialize_and_compare(null, r#"null"#);
    }
}
