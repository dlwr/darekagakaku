use serde_json::{json, Value};
use worker::d1::D1Database;
use worker::{Request, Response, Result, RouteContext};

use crate::agent_token::random_uuid;
use crate::agent_write;
use crate::agents;
use crate::db;
use crate::jsonrpc;
use crate::time::today_jst;

const PROTOCOL_VERSION: &str = "1.0";
const WRITE_SKILL: &str = "write-today";

const TASK_NOT_FOUND: i32 = -32001;
const UNSUPPORTED_OPERATION: i32 = -32004;
const VERSION_NOT_SUPPORTED: i32 = -32009;

pub fn agent_card(base_url: &str) -> Value {
    json!({
        "name": "誰かが書く日記",
        "description": "1日に1つの日記を誰でも書ける共有日記。今日の日記を読んで返すか、受け取った文章を今日の日記として保存する。言語モデルは持たず、決まった動作だけをする。",
        "version": env!("CARGO_PKG_VERSION"),
        "documentationUrl": format!("{base_url}/llms.txt"),
        "supportedInterfaces": [{
            "url": format!("{base_url}/a2a"),
            "protocolBinding": "JSONRPC",
            "protocolVersion": PROTOCOL_VERSION,
        }],
        "capabilities": { "streaming": false, "pushNotifications": false },
        "defaultInputModes": ["text/plain"],
        "defaultOutputModes": ["text/plain"],
        "securitySchemes": {
            "agentToken": { "httpAuthSecurityScheme": {
                "scheme": "Bearer",
                "description": format!("write-today にだけ必要。取り方は {base_url}/auth.md にある。"),
            } },
        },
        "skills": [
            {
                "id": "read-today",
                "name": "今日の日記を読む",
                "description": "SendMessage に対して今日の日記をMarkdownで返す。メッセージの内容は見ない。認証は要らない。",
                "tags": ["diary", "read"],
            },
            {
                "id": WRITE_SKILL,
                "name": "今日の日記を書く",
                "description": "message.metadata.skill に \"write-today\" を指定すると、メッセージのテキストを今日の日記として保存する（上書き）。Authorization: Bearer のトークンが要る。",
                "tags": ["diary", "write"],
            },
        ],
    })
}

fn message_text(params: &Value) -> String {
    params["message"]["parts"]
        .as_array()
        .map(|parts| {
            parts
                .iter()
                .filter_map(|part| part["text"].as_str())
                .collect::<Vec<_>>()
                .join("\n")
        })
        .unwrap_or_default()
}

fn wants_write(params: &Value) -> bool {
    params["message"]["metadata"]["skill"] == WRITE_SKILL
}

fn reply(text: &str, message_id: &str, context_id: &str) -> Value {
    json!({ "message": {
        "messageId": message_id,
        "contextId": context_id,
        "role": "ROLE_AGENT",
        "parts": [{ "text": text }],
    } })
}

/// POST /a2a - A2A（JSON-RPC）。タスクは作らず、SendMessage にその場でMessageを返す
pub async fn post(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let request: Value = match req.json().await {
        Ok(request) => request,
        Err(_) => {
            return Response::from_json(&jsonrpc::error(
                &Value::Null,
                jsonrpc::PARSE_ERROR,
                "Parse error",
            ));
        }
    };
    let id = &request["id"];

    // A2A-Version が無いリクエストは仕様上 0.3 として扱う。このサーバーは 1.0 だけを話す
    if req.headers().get("A2A-Version")?.as_deref() != Some(PROTOCOL_VERSION) {
        return Response::from_json(&jsonrpc::error(
            id,
            VERSION_NOT_SUPPORTED,
            "Only A2A-Version: 1.0 is supported",
        ));
    }

    let params = &request["params"];
    let response = match request["method"].as_str() {
        Some("SendMessage") => {
            let url = req.url()?;
            let base_url = format!("{}://{}", url.scheme(), url.host_str().unwrap_or("localhost"));

            let text = if wants_write(params) {
                let authorization = req.headers().get("Authorization")?;
                match agent_write::write_today(
                    &ctx.env,
                    authorization.as_deref(),
                    &message_text(params),
                )
                .await?
                {
                    Ok(_) => "今日の日記を保存した。".to_string(),
                    Err(refusal) => {
                        format!("保存しなかった: {} ({base_url}/auth.md)", refusal.message())
                    }
                }
            } else {
                let db: D1Database = ctx.env.d1("DB")?;
                let today = today_jst();
                let entry = db::get_entry(&db, &today).await?;
                agents::render_home_markdown(entry.as_ref(), &today, &base_url)
            };

            let context_id = params["message"]["contextId"]
                .as_str()
                .map(str::to_string)
                .unwrap_or_else(random_uuid);
            jsonrpc::result(id, reply(&text, &random_uuid(), &context_id))
        }
        Some("GetTask" | "CancelTask") => jsonrpc::error(id, TASK_NOT_FOUND, "Task not found"),
        Some(
            "SendStreamingMessage" | "SubscribeToTask" | "ListTasks" | "GetExtendedAgentCard",
        ) => jsonrpc::error(id, UNSUPPORTED_OPERATION, "This operation is not supported"),
        _ => jsonrpc::error(id, jsonrpc::METHOD_NOT_FOUND, "Method not found"),
    };

    Response::from_json(&response)
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASE: &str = "https://example.com";

    #[test]
    fn test_agent_card_interface_url() {
        assert_eq!(agent_card(BASE)["supportedInterfaces"][0]["url"], "https://example.com/a2a");
    }

    #[test]
    fn test_agent_card_interface_binding() {
        assert_eq!(agent_card(BASE)["supportedInterfaces"][0]["protocolBinding"], "JSONRPC");
    }

    #[test]
    fn test_agent_card_interface_protocol_version() {
        assert_eq!(agent_card(BASE)["supportedInterfaces"][0]["protocolVersion"], "1.0");
    }

    #[test]
    fn test_agent_card_has_required_top_level_fields() {
        let card = agent_card(BASE);
        for field in ["name", "description", "version", "capabilities"] {
            assert!(!card[field].is_null(), "missing {field}");
        }
    }

    #[test]
    fn test_agent_card_default_modes_are_plain_text() {
        let card = agent_card(BASE);
        assert_eq!(card["defaultInputModes"], json!(["text/plain"]));
        assert_eq!(card["defaultOutputModes"], json!(["text/plain"]));
    }

    #[test]
    fn test_agent_card_skill_ids() {
        let ids: Vec<String> = agent_card(BASE)["skills"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| s["id"].as_str().unwrap().to_string())
            .collect();
        assert_eq!(ids, ["read-today", WRITE_SKILL]);
    }

    #[test]
    fn test_agent_card_every_skill_has_required_fields() {
        for skill in agent_card(BASE)["skills"].as_array().unwrap() {
            for field in ["id", "name", "description", "tags"] {
                assert!(!skill[field].is_null(), "{} missing {field}", skill["id"]);
            }
        }
    }

    #[test]
    fn test_agent_card_does_not_claim_streaming() {
        assert_eq!(agent_card(BASE)["capabilities"]["streaming"], false);
    }

    #[test]
    fn test_message_text_joins_text_parts() {
        let params = json!({ "message": { "parts": [{ "text": "a" }, { "text": "b" }] } });
        assert_eq!(message_text(&params), "a\nb");
    }

    #[test]
    fn test_message_text_ignores_non_text_parts() {
        let params = json!({ "message": { "parts": [{ "url": "https://x" }, { "text": "b" }] } });
        assert_eq!(message_text(&params), "b");
    }

    #[test]
    fn test_message_text_empty_without_message() {
        assert_eq!(message_text(&json!({})), "");
    }

    #[test]
    fn test_wants_write_when_metadata_names_write_skill() {
        let params = json!({ "message": { "metadata": { "skill": "write-today" } } });
        assert!(wants_write(&params));
    }

    #[test]
    fn test_wants_write_false_without_metadata() {
        assert!(!wants_write(&json!({ "message": { "parts": [{ "text": "書いて" }] } })));
    }

    #[test]
    fn test_wants_write_false_for_other_skill() {
        let params = json!({ "message": { "metadata": { "skill": "read-today" } } });
        assert!(!wants_write(&params));
    }

    #[test]
    fn test_reply_role_is_agent() {
        assert_eq!(reply("hi", "m1", "c1")["message"]["role"], "ROLE_AGENT");
    }

    #[test]
    fn test_reply_carries_text_part() {
        assert_eq!(reply("hi", "m1", "c1")["message"]["parts"], json!([{ "text": "hi" }]));
    }

    #[test]
    fn test_reply_carries_message_id() {
        assert_eq!(reply("hi", "m1", "c1")["message"]["messageId"], "m1");
    }

    #[test]
    fn test_reply_carries_context_id() {
        assert_eq!(reply("hi", "m1", "c1")["message"]["contextId"], "c1");
    }
}
