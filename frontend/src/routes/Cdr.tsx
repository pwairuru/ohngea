import * as React from "react";
import { apiFetch } from "../lib/api";
import { DataTable } from "../components/DataTable";
import type { Column } from "../components/types";
import { PageHeader } from "../components/shell/PageHeader";
import { Button } from "../components/ui/button";
import { Label } from "../components/ui/label";
import { Input } from "../components/ui/input";
import { Card, CardContent, CardHeader, CardTitle } from "../components/ui/card";
import { toast, toastError } from "../lib/toast";
import { formatDateTime, formatDuration } from "../lib/format";
import { useAsyncAction } from "../lib/useAsyncAction";

const columns: Column<any>[] = [
  { key: "caller_id_number", header: "Caller", className: "num" },
  { key: "destination_number", header: "Destination", className: "num" },
  { key: "direction", header: "Dir" },
  { key: "duration", header: "Dur", className: "num", cell: (r) => formatDuration(r.duration) },
  { key: "hangup_cause", header: "Hangup" },
  { key: "start_stamp", header: "Start", cell: (r) => formatDateTime(r.start_stamp) },
];

export function Cdr() {
  const [summary, setSummary] = React.useState<Record<string, any> | null>(null);
  const [extension, setExtension] = React.useState("");
  const [appliedExtension, setAppliedExtension] = React.useState("");
  const [reloadKey, setReloadKey] = React.useState(0);

  React.useEffect(() => {
    apiFetch("/analytics/summary")
      .then((r: any) => setSummary(r ?? null))
      .catch(() => setSummary(null));
  }, [reloadKey]);

  const { run: doImport, pending } = useAsyncAction(async () => {
    try {
      await apiFetch("/cdr/import", { method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify({}) });
      toast("CDR import queued", { variant: "success" });
      setReloadKey((k) => k + 1);
    } catch (err) { toastError(err); }
  });

  const doExport = async () => {
    try {
      const qs = appliedExtension ? `?filter[extension]=${encodeURIComponent(appliedExtension)}` : "";
      const { API_BASE } = await import("../lib/api");
      const token = (() => { try { return localStorage.getItem("token"); } catch { return null; } })();
      const res = await fetch(`${API_BASE}/cdr/export${qs}`, {
        credentials: "include",
        headers: token ? { Authorization: `Bearer ${token}` } : {},
      });
      const ct = res.headers.get("content-type") || "";
      if (!res.ok || (!ct.includes("text/csv") && !ct.includes("text/plain") && !ct.includes("octet-stream"))) {
        const txt = await res.text();
        throw new Error(txt.slice(0, 300) || `export failed (${res.status})`);
      }
      const csv = await res.text();
      const blob = new Blob([csv], { type: "text/csv" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = "cdr-export.csv";
      a.click();
      URL.revokeObjectURL(url);
      toast("CDR exported", { variant: "success" });
    } catch (err) { toastError(err); }
  };

  const resource = appliedExtension ? `/cdr?filter[extension]=${encodeURIComponent(appliedExtension)}` : "/cdr";

  return (
    <>
      <PageHeader
        title="Call Detail Records"
        description="CDR history with analytics summary"
        actions={<><Button variant="outline" onClick={doExport}>Export CSV</Button><Button variant="outline" onClick={doImport} disabled={pending}>{pending ? "Queuing…" : "Queue Import"}</Button></>}
      />
      {summary && (
        <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
          {Object.entries(summary).slice(0, 8).map(([k, v]) => (
            <Card key={k}>
              <CardHeader><CardTitle className="text-xs uppercase text-muted-foreground">{k.replace(/_/g, " ")}</CardTitle></CardHeader>
              <CardContent className="text-2xl font-bold num">{typeof v === "object" ? JSON.stringify(v).slice(0, 40) : String(v)}</CardContent>
            </Card>
          ))}
        </div>
      )}
      <Card>
        <CardContent className="flex flex-wrap items-end gap-2 p-3">
          <div><Label>Extension filter</Label><Input value={extension} onChange={(e) => setExtension(e.target.value)} placeholder="1001" /></div>
          <Button variant="outline" onClick={() => { setAppliedExtension(extension); setReloadKey((k) => k + 1); }}>Apply</Button>
          <Button variant="ghost" onClick={() => { setExtension(""); setAppliedExtension(""); setReloadKey((k) => k + 1); }}>Clear</Button>
        </CardContent>
      </Card>
      <DataTable resource={resource} columns={columns} perPage={20} reloadKey={reloadKey} searchPlaceholder="Search CDR…" emptyTitle="No calls" emptyDescription="CDRs arrive via mod_xml_cdr" />
    </>
  );
}
