import * as React from "react";
import { DataTable } from "../components/DataTable";
import type { Column } from "../components/types";
import { PageHeader } from "../components/shell/PageHeader";

export function DomainLimits() {
  const [data, setData] = React.useState<any | null>(null);
  const [loading, setLoading] = React.useState(true);
  const load = React.useCallback(() => {
    setLoading(true);
    import("../lib/api").then(({ apiFetch }) =>
      apiFetch("/domain_limits")
        .then((r: any) => setData(r))
        .catch(() => setData(null))
        .finally(() => setLoading(false))
    );
  }, []);
  React.useEffect(() => { load(); }, [load]);
  if (loading) return <><PageHeader title="Domain Limits" description="Per-tenant resource limits" /><div className="p-4 text-sm text-muted-foreground">Loading…</div></>;
  if (!data) return <><PageHeader title="Domain Limits" description="Per-tenant resource limits" /><div className="p-4 text-sm text-muted-foreground">No data</div></>;
  const limits = data.limits || {};
  const rows = Object.entries(limits).map(([key, val]: any) => ({ key, ...(val as object) }));
  return (
    <>
      <PageHeader title="Domain Limits" description={`Per-tenant resource limits — domain ${data.domain_id?.slice(0, 8) ?? ""}`} />
      <div className="p-4 space-y-4">
        <div className="grid gap-3 md:grid-cols-3">
          {rows.length ? rows.map((r: any) => (
            <div key={r.key} className="rounded-lg border p-4 space-y-2">
              <div className="text-sm font-semibold capitalize">{r.key}</div>
              <div className="text-2xl font-bold">{r.used ?? 0} / {r.allowed ?? 1000}</div>
              <div className="h-2 w-full rounded bg-muted overflow-hidden"><div className="h-full bg-primary" style={{ width: `${Math.min(100, ((r.used ?? 0) / Math.max(1, r.allowed ?? 1000)) * 100)}%` }} /></div>
              <div className="text-xs text-muted-foreground">{r.remaining ?? 0} remaining</div>
            </div>
          )) : <div className="text-sm text-muted-foreground">No limits data</div>}
        </div>
      </div>
    </>
  );
}
