import * as React from "react";
import { flushSync } from "react-dom";
import { cn } from "./utils";

export type ToastVariant = "default" | "success" | "error";
type ToastItem = { id: number; title: string; description?: string; variant: ToastVariant };

let items: ToastItem[] = [];
let listeners: Array<() => void> = [];
let seq = 1;

function emit() {
  if (!listeners.length) return;
  flushSync(() => {
    listeners.forEach((l) => l());
  });
}

export function dismiss(id: number) {
  items = items.filter((t) => t.id !== id);
  emit();
}

export function clearToasts() {
  items = [];
  emit();
}

export function toast(title: string, opts: { description?: string; variant?: ToastVariant } = {}) {
  const id = seq++;
  items = [...items, { id, title, description: opts.description, variant: opts.variant ?? "default" }];
  emit();
  setTimeout(() => dismiss(id), 4000);
  return id;
}

export function toastError(err: unknown) {
  const msg = err instanceof Error ? err.message : String(err);
  return toast(msg.slice(0, 160), { variant: "error" });
}

export function Toaster() {
  const [, force] = React.useReducer((x: number) => x + 1, 0);
  const ref = React.useRef<(() => void) | null>(null);
  if (!ref.current) {
    ref.current = () => force();
    listeners.push(ref.current);
  }
  React.useLayoutEffect(() => {
    return () => {
      if (ref.current) listeners = listeners.filter((x) => x !== ref.current);
    };
  }, []);
  if (!items.length) return null;
  return (
    <div data-testid="toaster" className="fixed z-[70] flex flex-col gap-2 bottom-4 right-4 left-auto max-md:left-4">
      {items.map((t) => (
        <div
          key={t.id}
          data-testid="toast"
          role="status"
          className={cn(
            "relative rounded-md border bg-popover text-popover-foreground shadow-lg p-3 pr-8 max-md:max-w-none sm:max-w-sm",
            t.variant === "error" && "border-destructive/60",
            t.variant === "success" && "border-emerald-600/40"
          )}
        >
          <div className="text-sm font-medium">{t.title}</div>
          {t.description && <div className="text-muted-foreground mt-0.5">{t.description}</div>}
          <button aria-label="Dismiss" onClick={() => dismiss(t.id)} className="absolute right-2 top-1.5 text-muted-foreground hover:text-foreground">
            ✕
          </button>
        </div>
      ))}
    </div>
  );
}
