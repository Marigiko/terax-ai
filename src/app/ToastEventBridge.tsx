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

  return null;
}
