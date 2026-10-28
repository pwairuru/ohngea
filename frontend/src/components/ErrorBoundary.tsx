import * as React from "react";
import { useLocation } from "react-router-dom";

export class ErrorBoundary extends React.Component<{ children: React.ReactNode; resetKey?: string }, { error: Error | null }> {
  state = { error: null as Error | null };
  static getDerivedStateFromError(error: Error) {
    return { error };
  }
  componentDidUpdate(prevProps: { resetKey?: string }) {
    if (prevProps.resetKey !== this.props.resetKey && this.state.error) {
      this.setState({ error: null });
    }
  }
  render() {
    if (this.state.error) {
      return (
        <div className="p-8" data-testid="error-boundary">
          <div className="rounded-md border border-destructive bg-card p-4">
            <div className="font-mono text-sm font-semibold text-destructive">Unhandled error</div>
            <pre className="mt-2 whitespace-pre-wrap text-xs text-muted-foreground num">{String(this.state.error?.message ?? this.state.error)}</pre>
            <div className="mt-3 flex gap-2">
              <button className="inline-flex h-8 items-center rounded-md border px-3 text-sm hover:bg-accent" onClick={() => this.setState({ error: null })}>
                Try again
              </button>
              <button className="inline-flex h-8 items-center rounded-md border px-3 text-sm hover:bg-accent" onClick={() => window.location.reload()}>
                Reload
              </button>
            </div>
          </div>
        </div>
      );
    }
    return this.props.children;
  }
}

// Route-scoped boundary: resets automatically on navigation so one route's
// crash never poisons all other routes until manual reload.
export function RouteErrorBoundary({ children }: { children: React.ReactNode }) {
  const location = useLocation();
  return <ErrorBoundary resetKey={location.pathname} key={location.pathname}>{children}</ErrorBoundary>;
}
