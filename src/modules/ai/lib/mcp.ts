import { invoke } from "@tauri-apps/api/core";

export interface McpTool {
  name: string;
  description: string;
}

let cachedTools: McpTool[] = [];
let lastFetch = 0;
const CACHE_TTL = 30_000;

export async function listGatewayTools(
  gatewayUrl?: string,
  force = false,
): Promise<McpTool[]> {
  if (!force && Date.now() - lastFetch < CACHE_TTL && cachedTools.length > 0) {
    return cachedTools;
  }
  const result = await invoke<{ tools: McpTool[] }>("list_mcp_tools", {
    url: gatewayUrl,
  });
  cachedTools = result.tools ?? [];
  lastFetch = Date.now();
  return cachedTools;
}

export async function executeGatewayTool(
  toolName: string,
  input: Record<string, unknown>,
  gatewayUrl?: string,
): Promise<unknown> {
  return invoke<unknown>("execute_mcp_tool", {
    toolName,
    input,
    url: gatewayUrl,
  });
}

export function clearToolCache() {
  cachedTools = [];
  lastFetch = 0;
}
