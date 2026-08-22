import * as React from "react";
import { apiFetch } from "./api";

export type DomainApp = {
  app_name: string;
  enabled: boolean;
  category?: string;
  depends_on?: string[];
  description?: string;
  source?: string;
};

function decodeJwtPayload(token: string): Record<string, unknown> | null {
  try {
    const b64 = token.split(".")[1].replace(/-/g, "+").replace(/_/g, "/");
    const padded = b64 + "=".repeat((4 - (b64.length % 4)) % 4);
    return JSON.parse(atob(padded));
  } catch {
    return null;
  }
}

function resolveDomainId(explicit?: string): string | null {
  if (explicit) return explicit;
  const fromLs = typeof window !== "undefined" ? localStorage.getItem("domain_id") : null;
  if (fromLs) return fromLs;
  try {
    const token = typeof window !== "undefined" ? localStorage.getItem("token") : null;
    if (!token) return null;
    const payload = decodeJwtPayload(token);
    if (!payload) return null;
    if (payload?.domain_id) return String(payload.domain_id);
    if (payload?.domainId) return String(payload.domainId);
  } catch {
    // ignore
  }
  return null;
}

export async function fetchDomainApps(domainId: string): Promise<DomainApp[]> {
  const res: unknown = await apiFetch(`/domains/${domainId}/apps`);
  if (Array.isArray(res)) return res as DomainApp[];
  if (res && typeof res === "object" && "data" in (res as Record<string, unknown>)) {
    const data = (res as { data: unknown }).data;
    if (Array.isArray(data)) return data as DomainApp[];
  }
  return [];
}

export function useEnabledApps(domainId?: string): {
  enabled: Set<string>;
  loading: boolean;
  isEnabled: (app: string) => boolean;
} {
  const resolved = React.useMemo(() => resolveDomainId(domainId), [domainId]);
  const [enabled, setEnabled] = React.useState<Set<string> | null>(null);
  const [loading, setLoading] = React.useState<boolean>(!!resolved);

  React.useEffect(() => {
    if (!resolved) {
      setLoading(false);
      setEnabled(null);
      return;
    }
    let cancelled = false;
    setLoading(true);
    fetchDomainApps(resolved)
      .then((apps) => {
        if (cancelled) return;
        const s = new Set<string>();
        for (const a of apps) {
          if (a.enabled) s.add(a.app_name);
        }
        setEnabled(s);
      })
      .catch(() => {
        if (cancelled) return;
        // on error, treat as no filter (show all) — keep enabled null and let isEnabled return true
        setEnabled(null);
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, [resolved]);

  const isEnabled = React.useCallback(
    (app: string) => {
      if (loading) return true;
      if (enabled === null) return true;
      return enabled.has(app);
    },
    [enabled, loading],
  );

  return {
    enabled: enabled ?? new Set<string>(),
    loading,
    isEnabled,
  };
}
