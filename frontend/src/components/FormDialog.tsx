import * as React from "react";
import { Dialog, DialogHeader, DialogTitle } from "./ui/dialog";
import { Button } from "./ui/button";

export function FormDialog({ open, onOpenChange, title, children, onSubmit, pending, submitLabel = "Save", pendingLabel = "Saving…" }: {
  open: boolean;
  onOpenChange: (v: boolean) => void;
  title: string;
  children: React.ReactNode;
  onSubmit: () => void;
  pending?: boolean;
  submitLabel?: string;
  pendingLabel?: string;
}) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <form onSubmit={(e) => { e.preventDefault(); onSubmit(); }}>
        <DialogHeader><DialogTitle>{title}</DialogTitle></DialogHeader>
        <div className="space-y-3">{children}</div>
        <div className="mt-4 flex justify-end gap-2">
          <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>Cancel</Button>
          <Button type="submit" disabled={pending}>{pending ? pendingLabel : submitLabel}</Button>
        </div>
      </form>
    </Dialog>
  );
}
