import * as React from "react";
import { apiFetch } from "../lib/api";

type FsGlobal = {
  connected: boolean;
  host?: string;
  port?: number;
  latency_ms?: number | null;
  error?: string | null;
  timestamp?: string;
};

export function FsStatus() {
  const [status, setStatus] = React.useState<FsGlobal | null>(null);
  const [loading, setLoading] = React.useState(true);

  const fetchStatus = React.useCallback(async () => {
    try {
      const data = await apiFetch("/freeswitch/status");
      setStatus(data);
    } catch (e: any) {
      // apiFetch throws on 401 etc; treat as disconnected with error
      // if unauthorized, keep previous status or show disconnected
      const msg = e?.message || "fetch failed";
      // don't overwrite connected status with disconnected on transient 401? treat as error
      setStatus((prev) => ({
        connected: false,
        error: msg.slice(0, 200),
        host: prev?.host,
        port: prev?.port,
        latency_ms: null,
        timestamp: new Date().toISOString(),
      }));
    } finally {
      setLoading(false);
    }
  }, []);

  React.useEffect(() => {
    fetchStatus();
    const id = setInterval(fetchStatus, 30000);
    const onVis = () => {
      if (document.visibilityState === "visible") fetchStatus();
    };
    document.addEventListener("visibilitychange", onVis);
    return () => {
      clearInterval(id);
      document.removeEventListener("visibilitychange", onVis);
    };
  }, [fetchStatus]);

  const isConnected = status?.connected === true;
  const isChecking = loading && status === null;

  let dotColor = "bg-gray-400";
  let label = "Checking…";
  let aria = "checking freeswitch connection";
  if (!isChecking) {
    if (isConnected) {
      dotColor = "bg-green-500";
      label = status?.latency_ms != null ? `FS Connected · ${status.latency_ms}ms` : "FS Connected";
      aria = "freeswitch connected";
    } else {
      dotColor = "bg-red-500";
      label = "FS Disconnected";
      aria = "freeswitch disconnected";
    }
  }

  const tooltip = status
    ? [
        `FreeSWITCH ${isConnected ? "connected" : "disconnected"}`,
        status.host ? `host: ${status.host}:${status.port}` : null,
        status.latency_ms != null ? `latency: ${status.latency_ms}ms` : null,
        status.error ? `error: ${status.error}` : null,
        status.timestamp ? `checked: ${new Date(status.timestamp).toLocaleTimeString()}` : null,
      ]
        .filter(Boolean)
        .join("\n")
    : "Checking FreeSWITCH…";

  return (
    <span
      data-testid="fs-status"
      title={tooltip}
      aria-label={aria}
      className="inline-flex items-center gap-1.5 rounded-full border px-2.5 py-1 text-xs font-medium"
    >
      <span
        data-testid="fs-status-dot"
        aria-hidden
        className={`inline-block h-2 w-2 rounded-full ${dotColor} ${isConnected ? "shadow-[0_0_6px_rgba(34,197,94,0.6)]" : ""} ${isChecking ? "animate-pulse" : ""}`}
      />
      <span className="hidden sm:inline">{loading && status === null ? "Checking…" : label}</span>
      <span className="sm:hidden">{isChecking ? "…" : isConnected ? "●" : "○"}</span>
    </span>
  );
}
