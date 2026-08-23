import * as React from "react";
import { cn } from "../../lib/utils";

export function Sheet({ open, onOpenChange, side = "right", children }: { open: boolean; onOpenChange: (v: boolean) => void; side?: "left" | "right"; children: React.ReactNode }) {
  if (!open) return null;
  if (side === "left") {
    return (
      <div className="fixed inset-0 z-40 flex">
        <div className="w-80 max-w-[85vw] bg-background border-r p-4 overflow-auto">{children}</div>
        <div className="flex-1 bg-black/40" onClick={() => onOpenChange(false)} />
      </div>
    );
  }
  return (
    <div className="fixed inset-0 z-40 flex">
      <div className="flex-1 bg-black/40" onClick={() => onOpenChange(false)} />
      <div className="w-80 bg-background border-l p-4 overflow-auto">{children}</div>
    </div>
  );
}
