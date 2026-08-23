import * as React from "react";
import { apiFetch } from "../lib/api";
import { Card, CardContent, CardHeader, CardTitle } from "../components/ui/card";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "../components/ui/table";
import { Badge } from "../components/ui/badge";

const POLL_INTERVAL_MS = 3000; // active polling from FS, matches old 1980ms but less aggressive

export function ActiveCalls() {
  const [data, setData] = React.useState<any[]>([]);
  const [total, setTotal] = React.useState(0);
  const [loading, setLoading] = React.useState(false);
  const [lastUpdated, setLastUpdated] = React.useState<Date | null>(null);
  const [autoRefresh, setAutoRefresh] = React.useState(true);
  const [error, setError] = React.useState<string | null>(null);

  const load = React.useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const r: any = await apiFetch("/active_calls?per_page=100");
      setData(r.data || []);
      setTotal(r.total ?? r.data?.length ?? 0);
      setLastUpdated(new Date());
    } catch (e: any) {
      setError(e?.message || "Failed to load");
    } finally {
      setLoading(false);
    }
  }, []);

  React.useEffect(() => {
    load();
  }, [load]);

  React.useEffect(() => {
    if (!autoRefresh) return;
    const id = window.setInterval(() => {
      if (document.hidden) return;
      load();
    }, POLL_INTERVAL_MS);
    const onVis = () => {
      if (!document.hidden && autoRefresh) load();
    };
    document.addEventListener("visibilitychange", onVis);
    return () => {
      window.clearInterval(id);
      document.removeEventListener("visibilitychange", onVis);
    };
  }, [load, autoRefresh]);

  return (
    <div className="p-4 space-y-4">
      <div className="flex flex-wrap justify-between items-center gap-2">
        <h1 className="text-2xl font-bold">Active Calls</h1>
        <div className="flex items-center gap-2">
          <Badge data-testid="active-calls-count">{total} total</Badge>
          <span className={`h-2 w-2 rounded-full ${autoRefresh ? "bg-green-500 animate-pulse" : "bg-gray-300"}`} title={autoRefresh ? "Live polling" : "Paused"} />
          <button
            onClick={load}
            disabled={loading}
            className="rounded border px-3 py-1 text-sm hover:bg-muted disabled:opacity-50"
            data-testid="active-calls-refresh"
          >
            {loading ? "…" : "Refresh"}
          </button>
          <label className="flex items-center gap-1 text-sm">
            <input type="checkbox" checked={autoRefresh} onChange={(e) => setAutoRefresh(e.target.checked)} />
            Auto
          </label>
        </div>
      </div>
      {lastUpdated && (
        <p className="text-xs text-muted-foreground">
          Last updated {lastUpdated.toLocaleTimeString()} · polling every {POLL_INTERVAL_MS / 1000}s from FreeSWITCH
          {error && <span className="text-red-500"> · {error}</span>}
        </p>
      )}
      <Card>
        <CardHeader><CardTitle>Active Calls — live from FreeSWITCH (show channels)</CardTitle></CardHeader>
        <CardContent>
          <div className="hidden md:block">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>UUID</TableHead>
                  <TableHead>Direction</TableHead>
                  <TableHead>CID</TableHead>
                  <TableHead>Destination</TableHead>
                  <TableHead>State</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {data.map((d: any, i: number) => (
                  <TableRow key={d.call_uuid || i}>
                    <TableCell className="font-mono text-xs max-w-[180px] truncate" title={d.call_uuid}>{d.call_uuid || "—"}</TableCell>
                    <TableCell>
                      <span className={`inline-flex rounded px-2 py-0.5 text-xs ${d.direction === "inbound" ? "bg-blue-100 text-blue-800" : d.direction === "outbound" ? "bg-green-100 text-green-800" : "bg-gray-100"}`}>
                        {d.direction || "—"}
                      </span>
                    </TableCell>
                    <TableCell className="font-mono text-xs">{d.cid_num || d.cid_name || "—"}</TableCell>
                    <TableCell className="font-mono text-xs">{d.dest || "—"}</TableCell>
                    <TableCell className="text-xs">{d.state || "—"}</TableCell>
                  </TableRow>
                ))}
                {data.length === 0 && (
                  <TableRow>
                    <TableCell colSpan={5} className="text-center text-muted-foreground py-8">
                      No active calls — live view via ESL `show channels as json` (polling every {POLL_INTERVAL_MS / 1000}s)
                    </TableCell>
                  </TableRow>
                )}
              </TableBody>
            </Table>
          </div>
          <div className="md:hidden grid gap-3">
            {data.map((d: any, i: number) => (
              <Card key={d.call_uuid || i} className="p-3">
                <div className="flex justify-between text-xs">
                  <span className="font-mono truncate">{d.call_uuid || "—"}</span>
                  <span className="ml-2 shrink-0 rounded bg-muted px-2 py-0.5">{d.direction || "—"}</span>
                </div>
                <div className="mt-2 grid grid-cols-2 gap-2 text-sm">
                  <div><span className="text-muted-foreground">CID:</span> {d.cid_num || "—"}</div>
                  <div><span className="text-muted-foreground">Dest:</span> {d.dest || "—"}</div>
                  <div className="col-span-2"><span className="text-muted-foreground">State:</span> {d.state || "—"}</div>
                </div>
              </Card>
            ))}
            {data.length === 0 && <div className="text-center text-muted-foreground py-4">No active calls</div>}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
