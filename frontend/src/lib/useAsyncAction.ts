import * as React from "react";

export function useAsyncAction(fn: () => Promise<void>) {
  const [pending, setPending] = React.useState(false);
  const run = React.useCallback(async () => {
    setPending(true);
    try {
      await fn();
    } finally {
      setPending(false);
    }
  }, [fn]);
  return { run, pending };
}
