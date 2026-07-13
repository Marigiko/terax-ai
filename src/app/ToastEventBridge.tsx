import { displayAgent } from "@/modules/agents/lib/format";
import type { AgentSignal } from "@/modules/agents/lib/types";
import { usePreferencesStore } from "@/modules/settings/preferences";
import { useEffect } from "react";
import { toast } from "sonner";

/**
 * Bridges custom terax:toast events to the sonner toast system.
 * Used by slash commands (e.g. /mcp) to surface async results.
 */
export function ToastEventBridge() {
  useEffect(() => {
    const handler = (e: Event) => {
      const { message, variant } = (e as CustomEvent).detail;
      if (variant === "error") {
        toast.error(message);
      } else {
        toast(message);
      }
    };
    window.addEventListener("terax:toast", handler);
    return () => window.removeEventListener("terax:toast", handler);
  }, []);

  // Command-done toast: fires on every agent Stop turn when the user has
  // opted in via the `commandDoneToasts` preference. Mirrors the macOS
  // "Agent finished" bell but as a non-blocking toast and per-session.
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    import("@tauri-apps/api/event").then(({ listen }) => {
      listen<AgentSignal>("terax:agent-signal", (e) => {
        if (e.payload.kind !== "finished") return;
        if (!usePreferencesStore.getState().commandDoneToasts) return;
        import("@/modules/agents/store/agentStore").then(
          ({ useAgentStore }) => {
            const session = useAgentStore.getState().sessions[e.payload.id];
            if (!session) return;
            toast(`${displayAgent(session.agent)} finished`);
          },
        );
      }).then((u) => {
        unlisten = u;
      });
    });
    return () => {
      unlisten?.();
    };
  }, []);

  return null;
}
