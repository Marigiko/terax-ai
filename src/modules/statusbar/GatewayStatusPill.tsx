import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { useGatewayHealth } from "@/modules/ai/hooks/useGatewayHealth";

type Props = {
  url?: string;
  onRetry?: () => void;
};

export function GatewayStatusPill({ url = "http://localhost:8000", onRetry }: Props) {
  const { health, recheck } = useGatewayHealth(url);

  if (!health) return null;

  const color = health.healthy
    ? "bg-emerald-500/15 text-emerald-700 dark:text-emerald-400"
    : "bg-red-500/15 text-red-700 dark:text-red-400";

  const dot = health.healthy ? "bg-emerald-500" : "bg-red-500";
  const label = health.healthy ? `${health.latency_ms}ms` : "offline";

  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <button
          type="button"
          onClick={() => {
            recheck();
            onRetry?.();
          }}
          className={`flex shrink-0 cursor-pointer items-center gap-1 rounded-full px-2 py-0.5 text-[10.5px] font-medium transition-colors hover:opacity-80 ${color}`}
        >
          <span className={`h-1.5 w-1.5 rounded-full ${dot}`} />
          <span>Gateway {label}</span>
        </button>
      </TooltipTrigger>
      <TooltipContent side="top" className="text-[11px] leading-relaxed">
        {health.healthy ? (
          <>
            AI Workstation Gateway connected ({health.url})
            <br />
            Latency: {health.latency_ms}ms
          </>
        ) : (
          <>
            Gateway unreachable at {health.url}
            <br />
            {health.error ?? "Connection failed"}
            <br />
            <span className="text-muted-foreground">Click to retry</span>
          </>
        )}
      </TooltipContent>
    </Tooltip>
  );
}
