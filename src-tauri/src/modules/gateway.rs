use std::time::Duration;

use reqwest::Client;
use serde::{Deserialize, Serialize};

const GATEWAY_DEFAULT_URL: &str = "http://localhost:8000";
const HEALTH_TIMEOUT_SECS: u64 = 5;
const MEMORY_TIMEOUT_SECS: u64 = 10;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GatewayHealth {
    pub url: String,
    pub healthy: bool,
    pub latency_ms: u64,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MemoryEntry {
    pub id: String,
    pub content: String,
    pub scope: String,
    pub timestamp: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RecallResult {
    pub memories: Vec<MemoryEntry>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RememberRequest {
    pub content: String,
    pub scope: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct McpToolInfo {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct McpToolList {
    pub tools: Vec<McpToolInfo>,
}

/// Check if the AI Workstation gateway is healthy
#[tauri::command]
pub async fn check_gateway_health(url: Option<String>) -> Result<GatewayHealth, String> {
    let gateway_url = url.unwrap_or_else(|| GATEWAY_DEFAULT_URL.to_string());
    let health_url = format!("{}/health", gateway_url.trim_end_matches('/'));

    let client = Client::builder()
        .timeout(Duration::from_secs(HEALTH_TIMEOUT_SECS))
        .build()
        .map_err(|e| format!("failed to build http client: {e}"))?;

    let start = std::time::Instant::now();
    match client.get(&health_url).send().await {
        Ok(resp) => {
            let latency = start.elapsed().as_millis() as u64;
            let healthy = resp.status().is_success();
            Ok(GatewayHealth {
                url: gateway_url,
                healthy,
                latency_ms: latency,
                error: if healthy {
                    None
                } else {
                    Some(format!("gateway returned status {}", resp.status()))
                },
            })
        }
        Err(e) => {
            let latency = start.elapsed().as_millis() as u64;
            Ok(GatewayHealth {
                url: gateway_url,
                healthy: false,
                latency_ms: latency,
                error: Some(e.to_string()),
            })
        }
    }
}

/// Get the default gateway URL
#[tauri::command]
pub fn gateway_default_url() -> String {
    GATEWAY_DEFAULT_URL.to_string()
}

/// Recall semantic memories from the gateway
#[tauri::command]
pub async fn recall_memories(
    query: String,
    scope: Option<String>,
    url: Option<String>,
) -> Result<RecallResult, String> {
    let gateway_url = url.unwrap_or_else(|| GATEWAY_DEFAULT_URL.to_string());
    let recall_url = format!("{}/memory/recall", gateway_url.trim_end_matches('/'));

    let client = Client::builder()
        .timeout(Duration::from_secs(MEMORY_TIMEOUT_SECS))
        .build()
        .map_err(|e| format!("failed to build http client: {e}"))?;

    let body = serde_json::json!({
        "query": query,
        "scope": scope.as_deref().unwrap_or("project"),
    });

    let resp = client
        .post(&recall_url)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("memory recall request failed: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("memory recall returned status {}", resp.status()));
    }

    let result: RecallResult = resp
        .json()
        .await
        .map_err(|e| format!("failed to parse recall response: {e}"))?;

    Ok(result)
}

/// Remember a memory in the gateway
#[tauri::command]
pub async fn remember_memory(
    content: String,
    scope: Option<String>,
    url: Option<String>,
) -> Result<bool, String> {
    let gateway_url = url.unwrap_or_else(|| GATEWAY_DEFAULT_URL.to_string());
    let remember_url = format!("{}/memory/remember", gateway_url.trim_end_matches('/'));

    let client = Client::builder()
        .timeout(Duration::from_secs(MEMORY_TIMEOUT_SECS))
        .build()
        .map_err(|e| format!("failed to build http client: {e}"))?;

    let body = serde_json::json!({
        "content": content,
        "scope": scope.as_deref().unwrap_or("project"),
    });

    let resp = client
        .post(&remember_url)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("memory remember request failed: {e}"))?;

    Ok(resp.status().is_success())
}

/// List available MCP tools from the gateway
#[tauri::command]
pub async fn list_mcp_tools(url: Option<String>) -> Result<McpToolList, String> {
    let gateway_url = url.unwrap_or_else(|| GATEWAY_DEFAULT_URL.to_string());
    let tools_url = format!("{}/tools", gateway_url.trim_end_matches('/'));

    let client = Client::builder()
        .timeout(Duration::from_secs(HEALTH_TIMEOUT_SECS))
        .build()
        .map_err(|e| format!("failed to build http client: {e}"))?;

    let resp = client
        .get(&tools_url)
        .send()
        .await
        .map_err(|e| format!("failed to list MCP tools: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("tools endpoint returned status {}", resp.status()));
    }

    let result: McpToolList = resp
        .json()
        .await
        .map_err(|e| format!("failed to parse tools response: {e}"))?;

    Ok(result)
}

/// Execute an MCP tool on the gateway
#[tauri::command]
pub async fn execute_mcp_tool(
    tool_name: String,
    input: serde_json::Value,
    url: Option<String>,
) -> Result<serde_json::Value, String> {
    let gateway_url = url.unwrap_or_else(|| GATEWAY_DEFAULT_URL.to_string());
    let exec_url = format!(
        "{}/tools/{}/execute",
        gateway_url.trim_end_matches('/'),
        tool_name
    );

    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| format!("failed to build http client: {e}"))?;

    let resp = client
        .post(&exec_url)
        .json(&input)
        .send()
        .await
        .map_err(|e| format!("failed to execute MCP tool: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("tool execution returned status {}", resp.status()));
    }

    let result: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("failed to parse tool response: {e}"))?;

    Ok(result)
}
