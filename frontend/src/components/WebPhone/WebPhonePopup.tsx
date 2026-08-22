import * as React from "react";

export function WebPhonePopup({ open, onOpenChange }: { open: boolean; onOpenChange: (v: boolean) => void }) {
  if (!open) return null;
  return (
    <div data-testid="webphone-popup" className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
      <div className="bg-background p-4 rounded shadow max-w-sm w-full">
        <div className="flex justify-between items-center mb-2">
          <h3 className="font-semibold">WebPhone</h3>
          <button onClick={() => onOpenChange(false)} aria-label="Close webphone">✕</button>
        </div>
        <p className="text-sm text-muted-foreground">WebPhone popup stub (Task 4) — full dialpad in Task 5</p>
      </div>
    </div>
  );
}
