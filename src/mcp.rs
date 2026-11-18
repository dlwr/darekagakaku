use serde_json::{json, Value};
use worker::d1::D1Database;
use worker::{Env, Request, Response, Result, RouteContext};

use crate::agent_write;
use crate::agents;
use crate::db;
use crate::jsonrpc;
use crate::models::DiaryEntrySummary;
use crate::time::{is_valid_date, today_jst};

const SUPPORTED_PROTOCOL_VERSIONS: [&str; 3] = ["2025-11-25", "2025-06-18", "2025-03-26"];
const SERVER_NAME: &str = "darekagakaku";
const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");
const DESCRIPTION: &str =
    "「誰かが書く日記」を読み書きする。1日に1つの日記を誰でも書けて、日本時間で日付が変わると編集できなくなる。";

pub fn server_card(base_url: &str) -> Value {
    let endpoint = format!("{base_url}/mcp");
    json!({
        "serverInfo": { "name": SERVER_NAME, "version": SERVER_VERSION },
        "description": DESCRIPTION,
        "url": endpoint,
        "transport": { "type": "streamable-http", "endpoint": endpoint },
        "capabilities": { "tools": true },
    })
}

fn initialize_result(requested_version: Option<&str>) -> Value {
    let version = requested_version
        .filter(|v| SUPPORTED_PROTOCOL_VERSIONS.contains(v))
        .unwrap_or(SUPPORTED_PROTOCOL_VERSIONS[0]);

    json!({
        "protocolVersion": version,
        "capabilities": { "tools": {} },
        "serverInfo": { "name": SERVER_NAME, "version": SERVER_VERSION },
        "instructions": format!(
            "{DESCRIPTION} write_today は今日の日記を上書きするので、先に read_today で今の内容を確かめること。"
        ),
    })
}

fn tools_list() -> Value {
    let no_arguments = json!({ "type": "object", "properties": {} });
    json!({ "tools": [
        {
            "name": "read_today",
            "description": "今日の日記を読む。",
            "inputSchema": no_arguments,
        },
        {
            "name": "read_entry",
            "description": "指定した日の日記を読む。",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "date": { "type": "string", "description": "YYYY-MM-DD（日本時間）" },
                },
                "required": ["date"],
            },
        },
        {
            "name": "list_entries",
            "description": "過去の日記を新しい順に100件まで一覧する。",
            "inputSchema": no_arguments,
        },
        {
            "name": "write_today",
            "description": "今日の日記を上書きする。HTTPリクエストに Authorization: Bearer <トークン> が要る。トークンの取り方は /auth.md にある。",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "content": { "type": "string", "description": "本文。10000文字まで" },
                },
                "required": ["content"],
            },
        },
    ] })
}

fn tool_text(text: &str, is_error: bool) -> Value {
    json!({ "content": [{ "type": "text", "text": text }], "isError": is_error })
}

async fn call_tool(
    env: &Env,
    base_url: &str,
    authorization: Option<&str>,
    params: &Value,
) -> Result<Option<Value>> {
    let arguments = &params["arguments"];
    let db: D1Database = env.d1("DB")?;

    let result = match params["name"].as_str() {
        Some("read_today") => {
            let entry = db::get_entry(&db, &today_jst()).await?;
            tool_text(
                &agents::render_home_markdown(entry.as_ref(), &today_jst(), base_url),
                false,
            )
        }
        Some("read_entry") => match arguments["date"].as_str() {
            Some(date) if is_valid_date(date) => match db::get_entry(&db, date).await? {
                Some(entry) if !entry.is_blank() => {
                    tool_text(&agents::render_entry_markdown(&entry, base_url), false)
                }
                _ => tool_text("その日の日記は無い。", true),
            },
            _ => tool_text("date は YYYY-MM-DD で指定する。", true),
        },
        Some("list_entries") => {
            let summaries: Vec<DiaryEntrySummary> = db::list_past_entries(&db, 100)
                .await?
                .iter()
                .map(DiaryEntrySummary::from_entry)
                .collect();
            tool_text(&agents::render_archive_markdown(&summaries, base_url), false)
        }
        Some("write_today") => {
            let content = arguments["content"].as_str().unwrap_or_default();
            match agent_write::write_today(env, authorization, content).await? {
                Ok(_) => tool_text("今日の日記を保存した。", false),
                Err(refusal) => tool_text(
                    &format!("{} ({base_url}/auth.md)", refusal.message()),
                    true,
                ),
            }
        }
        _ => return Ok(None),
    };
    Ok(Some(result))
}

/// POST /mcp - MCPサーバー（Streamable HTTP、セッション無し、JSON応答のみ）
pub async fn post(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let message: Value = match req.json().await {
        Ok(message) => message,
        Err(_) => {
            return Response::from_json(&jsonrpc::error(
                &Value::Null,
                jsonrpc::PARSE_ERROR,
                "Parse error",
            ))
            .map(|r| r.with_status(400));
        }
    };

    let Some(method) = message["method"].as_str() else {
        return Response::from_json(&jsonrpc::error(
            &Value::Null,
            jsonrpc::INVALID_REQUEST,
            "Invalid Request",
        ))
        .map(|r| r.with_status(400));
    };

    // idの無いメッセージは通知。応答本文は返さない
    let Some(id) = message.get("id") else {
        return Ok(Response::empty()?.with_status(202));
    };

    let params = &message["params"];
    let result = match method {
        "initialize" => Some(initialize_result(params["protocolVersion"].as_str())),
        "ping" => Some(json!({})),
        "tools/list" => Some(tools_list()),
        "tools/call" => {
            let url = req.url()?;
            let base_url = format!("{}://{}", url.scheme(), url.host_str().unwrap_or("localhost"));
            let authorization = req.headers().get("Authorization")?;
            match call_tool(&ctx.env, &base_url, authorization.as_deref(), params).await? {
                Some(result) => Some(result),
                None => {
                    return Response::from_json(&jsonrpc::error(
                        id,
                        jsonrpc::INVALID_PARAMS,
                        "Unknown tool",
                    ));
                }
            }
        }
        _ => None,
    };

    Response::from_json(&match result {
        Some(result) => jsonrpc::result(id, result),
        None => jsonrpc::error(id, jsonrpc::METHOD_NOT_FOUND, "Method not found"),
    })
}

/// GET /mcp - サーバー発のストリームは提供しない
pub async fn get(_req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    let mut response = Response::error("Method Not Allowed", 405)?;
    response.headers_mut().set("Allow", "POST")?;
    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_card_has_server_name() {
        assert_eq!(server_card("https://example.com")["serverInfo"]["name"], "darekagakaku");
    }

    #[test]
    fn test_server_card_has_server_version() {
        assert!(server_card("https://example.com")["serverInfo"]["version"].is_string());
    }

    #[test]
    fn test_server_card_points_to_endpoint() {
        assert_eq!(
            server_card("https://example.com")["transport"]["endpoint"],
            "https://example.com/mcp"
        );
    }

    #[test]
    fn test_server_card_transport_is_streamable_http() {
        assert_eq!(
            server_card("https://example.com")["transport"]["type"],
            "streamable-http"
        );
    }

    #[test]
    fn test_server_card_declares_tools() {
        assert_eq!(server_card("https://example.com")["capabilities"]["tools"], true);
    }

    #[test]
    fn test_initialize_echoes_supported_version() {
        assert_eq!(
            initialize_result(Some("2025-03-26"))["protocolVersion"],
            "2025-03-26"
        );
    }

    #[test]
    fn test_initialize_falls_back_to_latest_for_unknown_version() {
        assert_eq!(
            initialize_result(Some("1999-01-01"))["protocolVersion"],
            SUPPORTED_PROTOCOL_VERSIONS[0]
        );
    }

    #[test]
    fn test_initialize_falls_back_to_latest_without_version() {
        assert_eq!(
            initialize_result(None)["protocolVersion"],
            SUPPORTED_PROTOCOL_VERSIONS[0]
        );
    }

    #[test]
    fn test_initialize_declares_tools_capability() {
        assert!(initialize_result(None)["capabilities"]["tools"].is_object());
    }

    #[test]
    fn test_tools_list_names() {
        let names: Vec<String> = tools_list()["tools"]
            .as_array()
            .unwrap()
            .iter()
            .map(|t| t["name"].as_str().unwrap().to_string())
            .collect();
        assert_eq!(names, ["read_today", "read_entry", "list_entries", "write_today"]);
    }

    #[test]
    fn test_tools_list_every_tool_has_object_input_schema() {
        for tool in tools_list()["tools"].as_array().unwrap() {
            assert_eq!(tool["inputSchema"]["type"], "object", "{}", tool["name"]);
        }
    }

    #[test]
    fn test_tools_list_write_today_requires_content() {
        assert_eq!(tools_list()["tools"][3]["inputSchema"]["required"], json!(["content"]));
    }

    #[test]
    fn test_tool_text_wraps_text_content() {
        assert_eq!(
            tool_text("hi", false)["content"],
            json!([{ "type": "text", "text": "hi" }])
        );
    }

    #[test]
    fn test_tool_text_marks_errors() {
        assert_eq!(tool_text("no", true)["isError"], true);
    }
}
