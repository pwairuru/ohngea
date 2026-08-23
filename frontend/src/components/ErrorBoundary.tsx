import * as React from "react";

export class ErrorBoundary extends React.Component<{ children: React.ReactNode }, { error: Error | null }> {
  state = { error: null as Error | null };
  static getDerivedStateFromError(error: Error) {
    return { error };
  }
  render() {
    if (this.state.error) {
      return (
        <div className="p-8" data-testid="error-boundary">
          <div className="rounded-md border border-destructive bg-card p-4">
            <div className="font-mono text-sm font-semibold text-destructive">Unhandled error</div>
            <pre className="mt-2 whitespace-pre-wrap text-xs text-muted-foreground num">{String(this.state.error?.message ?? this.state.error)}</pre>
            <button className="mt-3 inline-flex h-8 items-center rounded-md border px-3 text-sm hover:bg-accent" onClick={() => window.location.reload()}>
              Reload
            </button>
          </div>
        </div>
      );
    }
    return this.props.children;
  }
}
