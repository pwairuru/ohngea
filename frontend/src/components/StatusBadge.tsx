import * as React from "react";
import { cn } from "../lib/utils";

const tones: Record<string, string> = {
  ok: "bg-emerald-500",
  warn: "bg-amber-500",
  error: "bg-red-500",
  off: "bg-zinc-400 dark:bg-zinc-600",
};

export function StatusBadge({ tone, children }: { tone: keyof typeof tones; children: React.ReactNode }) {
  return (
    <span className="inline-flex items-center gap-1.5 text-xs">
      <span className={cn("inline-block h-1.5 w-1.5 rounded-full", tones[tone])} />
      {children}
    </span>
  );
}
