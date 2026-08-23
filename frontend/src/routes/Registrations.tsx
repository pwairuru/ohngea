import * as React from "react";
import { apiFetch } from "../lib/api";
import { Card, CardContent, CardHeader, CardTitle } from "../components/ui/card";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "../components/ui/table";
import { Badge } from "../components/ui/badge";

const POLL_INTERVAL_MS = 5000; // registrations less volatile than calls

export function Registrations() {
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
      const r: any = await apiFetch("/registrations?per_page=100");
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
        <h1 className="text-2xl font-bold">Registrations</h1>
        <div className="flex items-center gap-2">
          <Badge data-testid="regs-count">{total} total</Badge>
          <span className={`h-2 w-2 rounded-full ${autoRefresh ? "bg-green-500 animate-pulse" : "bg-gray-300"}`} title={autoRefresh ? "Live polling" : "Paused"} />
          <button
            onClick={load}
            disabled={loading}
            className="rounded border px-3 py-1 text-sm hover:bg-muted disabled:opacity-50"
            data-testid="regs-refresh"
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
          Last updated {lastUpdated.toLocaleTimeString()} · polling every {POLL_INTERVAL_MS / 1000}s from FreeSWITCH (sofia reg)
          {error && <span className="text-red-500"> · {error}</span>}
        </p>
      )}
      <Card>
        <CardHeader><CardTitle>Registrations — live from FreeSWITCH</CardTitle></CardHeader>
        <CardContent>
          <div className="hidden md:block">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>User</TableHead>
                  <TableHead>Realm</TableHead>
                  <TableHead>Contact / IP</TableHead>
                  <TableHead>Status</TableHead>
                  <TableHead>Expires</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {data.map((d: any, i: number) => (
                  <TableRow key={`${d.reg_user}-${d.realm}-${i}`}>
                    <TableCell className="font-mono text-xs">{d.reg_user || d.user || "—"}</TableCell>
                    <TableCell className="text-xs">{d.realm || "—"}</TableCell>
                    <TableCell className="font-mono text-xs max-w-[220px] truncate" title={d.network_ip || d.contact}>
                      {d.network_ip || d.contact || "—"}
                    </TableCell>
                    <TableCell className="text-xs max-w-[220px] truncate" title={d.status}>{d.status || "Registered"}</TableCell>
                    <TableCell className="text-xs">{d.expires != null ? `${d.expires}s` : "—"}</TableCell>
                  </TableRow>
                ))}
                {data.length === 0 && (
                  <TableRow>
                    <TableCell colSpan={5} className="text-center text-muted-foreground py-8">
                      No registrations — live view via ESL `show registrations as json` / `sofia xmlstatus reg` (polling every {POLL_INTERVAL_MS / 1000}s)
                    </TableCell>
                  </TableRow>
                )}
              </TableBody>
            </Table>
          </div>
          <div className="md:hidden grid gap-3">
            {data.map((d: any, i: number) => (
              <Card key={`${d.reg_user}-${i}`} className="p-3">
                <div className="font-mono text-sm">{d.reg_user || "—"}@{d.realm || "—"}</div>
                <div className="mt-1 text-xs text-muted-foreground truncate">{d.network_ip || d.contact || "—"}</div>
                <div className="mt-2 flex justify-between text-xs">
                  <span className="rounded bg-muted px-2 py-0.5 truncate max-w-[60%]">{d.status || "Registered"}</span>
                  <span>{d.expires != null ? `${d.expires}s` : "—"}</span>
                </div>
              </Card>
            ))}
            {data.length === 0 && <div className="text-center text-muted-foreground py-4">No registrations</div>}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
