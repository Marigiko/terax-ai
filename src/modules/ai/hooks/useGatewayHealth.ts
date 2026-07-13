import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState, useCallback } from "react";

export type GatewayHealth = {
  url: string;
  healthy: boolean;
  latency_ms: number;
  error: string | null;
};

const DEFAULT_GATEWAY_URL = "http://localhost:8000";
const POLL_INTERVAL_MS = 10_000;

export function useGatewayHealth(
  gatewayUrl: string = DEFAULT_GATEWAY_URL,
  enabled: boolean = true,
) {
  const [health, setHealth] = useState<GatewayHealth | null>(null);
  const [loading, setLoading] = useState(false);

  const check = useCallback(async () => {
    setLoading(true);
    try {
      const result = await invoke<GatewayHealth>("check_gateway_health", {
        url: gatewayUrl,
      });
      setHealth(result);
    } catch (e) {
      setHealth({
        url: gatewayUrl,
        healthy: false,
        latency_ms: 0,
        error: String(e),
      });
    } finally {
      setLoading(false);
    }
  }, [gatewayUrl]);

  useEffect(() => {
    if (!enabled) return;
    check();
    const interval = setInterval(check, POLL_INTERVAL_MS);
    return () => clearInterval(interval);
  }, [enabled, check]);

  return { health, loading, recheck: check };
}
