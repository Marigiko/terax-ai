import {
  CommandDialog,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "@/components/ui/command";
import { native } from "@/modules/ai/lib/native";
import { useEffect, useState } from "react";

type SshHost = {
  name: string;
  dest: string;
};

/**
 * Summarize an ssh hostlabel into a [<user>@]<host> destination for display.
 * Real host lines can carry User / HostName directives, but we only parse
 * the alias name here — the actual address lives in the user's ~/.ssh/config.
 */
function hostToDisplay(h: string): { name: string; dest: string } {
  return { name: h, dest: h };
}

export function MultiSshDialog({
  open,
  onOpenChange,
  onSelect,
}: {
  open: boolean;
  onOpenChange: (v: boolean) => void;
  onSelect: (host: string) => void;
}) {
  const [hosts, setHosts] = useState<SshHost[]>([]);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (!open) return;
    let alive = true;
    setLoading(true);
    native
      .sshListHosts()
      .then((list) => {
        if (!alive) return;
        setHosts(list.map((h) => hostToDisplay(h)));
      })
      .catch(() => {
        if (alive) setHosts([]);
      })
      .finally(() => {
        if (alive) setLoading(false);
      });
    return () => {
      alive = false;
    };
  }, [open]);

  return (
    <CommandDialog
      open={open}
      onOpenChange={onOpenChange}
      title="SSH hosts"
      description="Pick a connection from ~/.ssh/config to open a terminal to."
    >
      <CommandInput placeholder="Filter hosts…" />
      <CommandList>
        <CommandEmpty>
          {loading ? "Loading…" : "No SSH hosts found in ~/.ssh/config."}
        </CommandEmpty>
        <CommandGroup heading="Hosts">
          {hosts.map((h) => (
            <CommandItem
              key={h.name}
              value={`${h.name} ${h.dest}`}
              onSelect={() => {
                onSelect(h.name);
                onOpenChange(false);
              }}
            >
              <span className="font-mono text-xs">{h.name}</span>
              {h.dest !== h.name && (
                <span className="ml-2 text-xs text-muted-foreground">
                  ({h.dest})
                </span>
              )}
            </CommandItem>
          ))}
        </CommandGroup>
      </CommandList>
    </CommandDialog>
  );
}
