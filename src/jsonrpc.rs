use serde_json::{json, Value};

pub const METHOD_NOT_FOUND: i32 = -32601;
pub const INVALID_REQUEST: i32 = -32600;
pub const INVALID_PARAMS: i32 = -32602;
pub const PARSE_ERROR: i32 = -32700;

pub fn result(id: &Value, result: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "result": result })
}

pub fn error(id: &Value, code: i32, message: &str) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_result_echoes_id() {
        assert_eq!(result(&json!(7), json!({}))["id"], 7);
    }

    #[test]
    fn test_result_carries_result() {
        assert_eq!(result(&json!(7), json!({"a": 1}))["result"]["a"], 1);
    }

    #[test]
    fn test_result_declares_jsonrpc_version() {
        assert_eq!(result(&json!(7), json!({}))["jsonrpc"], "2.0");
    }

    #[test]
    fn test_error_carries_code() {
        assert_eq!(error(&json!("x"), -32601, "Method not found")["error"]["code"], -32601);
    }

    #[test]
    fn test_error_carries_message() {
        assert_eq!(
            error(&json!("x"), -32601, "Method not found")["error"]["message"],
            "Method not found"
        );
    }

    #[test]
    fn test_error_has_no_result() {
        assert!(error(&json!("x"), -32601, "Method not found").get("result").is_none());
    }
}
