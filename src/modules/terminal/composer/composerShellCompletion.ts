import type { CompletionContext, CompletionResult } from "@codemirror/autocomplete";

const COMMON_COMMANDS = [
  "cd", "ls", "cat", "echo", "grep", "find", "rm", "cp", "mv", "mkdir",
  "git", "npm", "pnpm", "bun", "cargo", "python", "pip", "docker",
  "kubectl", "terraform", "aws", "gcloud", "ssh", "curl", "wget",
];

export function composerShellCompletionSource(
  context: CompletionContext,
): CompletionResult | null {
  const word = context.matchBefore(/[\w-]*/);
  if (!word || (word.from === word.to && !context.explicit)) return null;

  const options = COMMON_COMMANDS.map((cmd) => ({
    label: cmd,
    type: "keyword" as const,
    boost: -1,
  }));

  return {
    from: word.from,
    options,
    validFor: /^[\w-]*$/,
  };
}

export function shellCompletionOptions(prefix: string): string[] {
  const p = prefix.toLowerCase();
  return COMMON_COMMANDS.filter((cmd) => cmd.startsWith(p));
}
