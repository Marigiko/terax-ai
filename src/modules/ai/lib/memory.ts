import { invoke } from "@tauri-apps/api/core";

export type MemoryScope = "global" | "project" | "session";

export interface Memory {
  id: string;
  content: string;
  scope: MemoryScope;
  timestamp: string;
}

export async function recall(
  query: string,
  scope: MemoryScope = "project",
  gatewayUrl?: string,
): Promise<Memory[]> {
  const result = await invoke<{ memories: Memory[] }>("recall_memories", {
    query,
    scope,
    url: gatewayUrl,
  });
  return result.memories ?? [];
}

export async function remember(
  content: string,
  scope: MemoryScope = "project",
  gatewayUrl?: string,
): Promise<boolean> {
  return invoke<boolean>("remember_memory", {
    content,
    scope,
    url: gatewayUrl,
  });
}

/** Build a system-prompt block from recalled memories. */
export function formatMemoriesBlock(memories: Memory[]): string {
  if (memories.length === 0) return "";
  const lines = memories.map((m, i) => `[${i + 1}] ${m.content.trim()}`);
  return `## PROJECT MEMORY (from AI Workstation Gateway)\n${lines.join("\n")}`;
}
