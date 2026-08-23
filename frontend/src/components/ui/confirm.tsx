import * as React from "react";
import { Dialog } from "./dialog";
import { Button } from "./button";

export type ConfirmOptions = { title: string; description?: string; confirmLabel?: string; cancelLabel?: string };
type ConfirmFn = (opts: ConfirmOptions) => Promise<boolean>;

const ConfirmContext = React.createContext<ConfirmFn>(async () => false);

export function useConfirm() {
  return React.useContext(ConfirmContext);
}

export function ConfirmProvider({ children }: { children: React.ReactNode }) {
  const [state, setState] = React.useState<ConfirmOptions | null>(null);
  const resolver = React.useRef<((v: boolean) => void) | null>(null);
  const confirm = React.useCallback<ConfirmFn>((opts) => {
    setState(opts);
    return new Promise<boolean>((resolve) => {
      resolver.current = resolve;
    });
  }, []);
  const close = (v: boolean) => {
    setState(null);
    resolver.current?.(v);
    resolver.current = null;
  };
  return (
    <ConfirmContext.Provider value={confirm}>
      {children}
      <Dialog open={state !== null} onOpenChange={(o) => !o && close(false)}>
        {state && (
          <div data-testid="confirm-dialog">
            <h3 className="text-lg font-semibold">{state.title}</h3>
            {state.description && <p className="text-sm text-muted-foreground mt-1">{state.description}</p>}
            <div className="flex justify-end gap-2 mt-6">
              <Button variant="outline" onClick={() => close(false)} data-testid="confirm-cancel">{state.cancelLabel ?? "Cancel"}</Button>
              <Button variant="destructive" onClick={() => close(true)} data-testid="confirm-ok">{state.confirmLabel ?? "Confirm"}</Button>
            </div>
          </div>
        )}
      </Dialog>
    </ConfirmContext.Provider>
  );
}
