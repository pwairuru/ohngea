import * as React from "react";
export function Sheet({ open, onOpenChange, children }: { open: boolean; onOpenChange: (v: boolean) => void; children: React.ReactNode }) {
  if (!open) return null;
  return (
    <div className="fixed inset-0 z-40 flex">
      <div className="flex-1 bg-black/40" onClick={() => onOpenChange(false)} />
      <div className="w-80 bg-background border-l p-4 overflow-auto">{children}</div>
    </div>
  );
}
