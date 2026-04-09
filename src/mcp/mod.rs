/// MCP (Model Context Protocol) server implementation for ztlgr.
///
/// Implements the MCP specification (2025-03-26) over stdio transport,
/// exposing grimoire operations as tools that LLM agents can invoke.
///
/// Protocol: JSON-RPC 2.0 over stdin/stdout, newline-delimited.
pub mod server;
pub mod tools;

use serde::{Deserialize, Serialize};

// MCP protocol version we support
pub const PROTOCOL_VERSION: &str = "2025-03-26";

// Server info
pub const SERVER_NAME: &str = "ztlgr";

// --- JSON-RPC 2.0 types ---

/// A JSON-RPC 2.0 request/notification from the client.
#[derive(Debug, Deserialize)]
pub struct JsonRpcMessage {
    pub jsonrpc: String,
    #[serde(default)]
    pub id: Option<serde_json::Value>,
    pub method: Option<String>,
    pub params: Option<serde_json::Value>,
    /// For responses from client (e.g., to server requests)
    pub result: Option<serde_json::Value>,
    pub error: Option<JsonRpcError>,
}

/// A JSON-RPC 2.0 response.
#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

impl JsonRpcResponse {
    pub fn success(id: serde_json::Value, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn error(id: serde_json::Value, error: JsonRpcError) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(error),
        }
    }
}

/// A JSON-RPC 2.0 error object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl JsonRpcError {
    pub fn method_not_found(method: &str) -> Self {
        Self {
            code: -32601,
            message: format!("Method not found: {}", method),
            data: None,
        }
    }

    pub fn invalid_params(msg: &str) -> Self {
        Self {
            code: -32602,
            message: msg.to_string(),
            data: None,
        }
    }

    pub fn internal_error(msg: &str) -> Self {
        Self {
            code: -32603,
            message: msg.to_string(),
            data: None,
        }
    }

    pub fn parse_error(msg: &str) -> Self {
        Self {
            code: -32700,
            message: msg.to_string(),
            data: None,
        }
    }
}

// --- MCP Initialization types ---

/// Client capabilities sent during initialization.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeParams {
    pub protocol_version: String,
    #[serde(default)]
    pub capabilities: serde_json::Value,
    #[serde(default)]
    pub client_info: Option<ClientInfo>,
}

#[derive(Debug, Deserialize)]
pub struct ClientInfo {
    pub name: String,
    #[serde(default)]
    pub version: Option<String>,
}

/// Server response to initialization.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeResult {
    pub protocol_version: String,
    pub capabilities: ServerCapabilities,
    pub server_info: ServerInfo,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ServerCapabilities {
    pub tools: ToolsCapability,
}

#[derive(Debug, Serialize)]
pub struct ToolsCapability {
    #[serde(rename = "listChanged")]
    pub list_changed: bool,
}

#[derive(Debug, Serialize)]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
}

// --- Tool types ---

/// Tool definition returned in tools/list response.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

/// Result of a tools/call invocation.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolCallResult {
    pub content: Vec<ToolContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
}

impl ToolCallResult {
    pub fn text(text: String) -> Self {
        Self {
            content: vec![ToolContent::Text { text }],
            is_error: None,
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            content: vec![ToolContent::Text { text: message }],
            is_error: Some(true),
        }
    }
}

/// Content items in a tool result.
#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ToolContent {
    #[serde(rename = "text")]
    Text { text: String },
}

/// Parameters for tools/call request.
#[derive(Debug, Deserialize)]
pub struct ToolCallParams {
    pub name: String,
    #[serde(default)]
    pub arguments: Option<serde_json::Value>,
}

/// Parameters for tools/list request (pagination).
#[derive(Debug, Deserialize)]
pub struct ToolListParams {
    #[serde(default)]
    pub cursor: Option<String>,
}

/// Response body for tools/list.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolListResult {
    pub tools: Vec<ToolDefinition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jsonrpc_response_success() {
        let resp =
            JsonRpcResponse::success(serde_json::json!(1), serde_json::json!({"key": "value"}));
        assert_eq!(resp.jsonrpc, "2.0");
        assert_eq!(resp.id, serde_json::json!(1));
        assert!(resp.result.is_some());
        assert!(resp.error.is_none());
    }

    #[test]
    fn test_jsonrpc_response_error() {
        let err = JsonRpcError::method_not_found("unknown");
        let resp = JsonRpcResponse::error(serde_json::json!(2), err);
        assert_eq!(resp.jsonrpc, "2.0");
        assert!(resp.result.is_none());
        let e = resp.error.unwrap();
        assert_eq!(e.code, -32601);
        assert!(e.message.contains("unknown"));
    }

    #[test]
    fn test_jsonrpc_error_codes() {
        let err = JsonRpcError::parse_error("bad json");
        assert_eq!(err.code, -32700);

        let err = JsonRpcError::invalid_params("missing field");
        assert_eq!(err.code, -32602);

        let err = JsonRpcError::internal_error("oops");
        assert_eq!(err.code, -32603);

        let err = JsonRpcError::method_not_found("foo");
        assert_eq!(err.code, -32601);
    }

    #[test]
    fn test_jsonrpc_response_serialization() {
        let resp = JsonRpcResponse::success(serde_json::json!(1), serde_json::json!({"ok": true}));
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"id\":1"));
        assert!(json.contains("\"ok\":true"));
        // error should not appear when None
        assert!(!json.contains("\"error\""));
    }

    #[test]
    fn test_jsonrpc_error_serialization() {
        let err = JsonRpcError::internal_error("server crash");
        let resp = JsonRpcResponse::error(serde_json::json!(5), err);
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"error\""));
        assert!(json.contains("-32603"));
        assert!(json.contains("server crash"));
        // result should not appear when None
        assert!(!json.contains("\"result\""));
    }

    #[test]
    fn test_tool_call_result_text() {
        let result = ToolCallResult::text("hello".to_string());
        assert_eq!(result.content.len(), 1);
        assert!(result.is_error.is_none());
        let json = serde_json::to_value(&result).unwrap();
        let content = &json["content"][0];
        assert_eq!(content["type"], "text");
        assert_eq!(content["text"], "hello");
    }

    #[test]
    fn test_tool_call_result_error() {
        let result = ToolCallResult::error("something failed".to_string());
        assert_eq!(result.is_error, Some(true));
        let json = serde_json::to_value(&result).unwrap();
        assert_eq!(json["isError"], true);
    }

    #[test]
    fn test_initialize_params_deserialization() {
        let json = serde_json::json!({
            "protocolVersion": "2025-03-26",
            "capabilities": {},
            "clientInfo": {
                "name": "TestClient",
                "version": "1.0"
            }
        });
        let params: InitializeParams = serde_json::from_value(json).unwrap();
        assert_eq!(params.protocol_version, "2025-03-26");
        let info = params.client_info.unwrap();
        assert_eq!(info.name, "TestClient");
        assert_eq!(info.version, Some("1.0".to_string()));
    }

    #[test]
    fn test_initialize_params_minimal() {
        let json = serde_json::json!({
            "protocolVersion": "2025-03-26"
        });
        let params: InitializeParams = serde_json::from_value(json).unwrap();
        assert_eq!(params.protocol_version, "2025-03-26");
        assert!(params.client_info.is_none());
    }

    #[test]
    fn test_initialize_result_serialization() {
        let result = InitializeResult {
            protocol_version: PROTOCOL_VERSION.to_string(),
            capabilities: ServerCapabilities {
                tools: ToolsCapability {
                    list_changed: false,
                },
            },
            server_info: ServerInfo {
                name: SERVER_NAME.to_string(),
                version: "0.5.0".to_string(),
            },
            instructions: Some("Test instructions".to_string()),
        };
        let json = serde_json::to_value(&result).unwrap();
        assert_eq!(json["protocolVersion"], "2025-03-26");
        assert_eq!(json["serverInfo"]["name"], "ztlgr");
        assert_eq!(json["capabilities"]["tools"]["listChanged"], false);
        assert_eq!(json["instructions"], "Test instructions");
    }

    #[test]
    fn test_initialize_result_without_instructions() {
        let result = InitializeResult {
            protocol_version: PROTOCOL_VERSION.to_string(),
            capabilities: ServerCapabilities {
                tools: ToolsCapability {
                    list_changed: false,
                },
            },
            server_info: ServerInfo {
                name: SERVER_NAME.to_string(),
                version: "0.5.0".to_string(),
            },
            instructions: None,
        };
        let json = serde_json::to_value(&result).unwrap();
        assert!(json.get("instructions").is_none());
    }

    #[test]
    fn test_tool_definition_serialization() {
        let tool = ToolDefinition {
            name: "search".to_string(),
            description: "Search notes".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string" }
                },
                "required": ["query"]
            }),
        };
        let json = serde_json::to_value(&tool).unwrap();
        assert_eq!(json["name"], "search");
        assert_eq!(json["inputSchema"]["type"], "object");
    }

    #[test]
    fn test_tool_list_result_serialization() {
        let result = ToolListResult {
            tools: vec![],
            next_cursor: None,
        };
        let json = serde_json::to_value(&result).unwrap();
        assert_eq!(json["tools"], serde_json::json!([]));
        assert!(json.get("nextCursor").is_none());
    }

    #[test]
    fn test_tool_call_params_deserialization() {
        let json = serde_json::json!({
            "name": "search",
            "arguments": { "query": "rust" }
        });
        let params: ToolCallParams = serde_json::from_value(json).unwrap();
        assert_eq!(params.name, "search");
        let args = params.arguments.unwrap();
        assert_eq!(args["query"], "rust");
    }

    #[test]
    fn test_tool_call_params_without_arguments() {
        let json = serde_json::json!({
            "name": "list_notes"
        });
        let params: ToolCallParams = serde_json::from_value(json).unwrap();
        assert_eq!(params.name, "list_notes");
        assert!(params.arguments.is_none());
    }

    #[test]
    fn test_jsonrpc_message_deserialization_request() {
        let json = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/list",
            "params": {}
        });
        let msg: JsonRpcMessage = serde_json::from_value(json).unwrap();
        assert_eq!(msg.jsonrpc, "2.0");
        assert_eq!(msg.id, Some(serde_json::json!(1)));
        assert_eq!(msg.method, Some("tools/list".to_string()));
    }

    #[test]
    fn test_jsonrpc_message_deserialization_notification() {
        let json = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized"
        });
        let msg: JsonRpcMessage = serde_json::from_value(json).unwrap();
        assert!(msg.id.is_none());
        assert_eq!(msg.method, Some("notifications/initialized".to_string()));
    }

    #[test]
    fn test_protocol_version_constant() {
        assert_eq!(PROTOCOL_VERSION, "2025-03-26");
    }

    #[test]
    fn test_server_name_constant() {
        assert_eq!(SERVER_NAME, "ztlgr");
    }
}
