import * as React from "react";
import { Card, CardContent, CardHeader, CardTitle } from "../components/ui/card";
import { apiFetch } from "../lib/api";
import {
  BarChart, Bar, XAxis, YAxis, Tooltip, ResponsiveContainer, CartesianGrid, PieChart, Pie, Cell, Legend,
} from "recharts";

function formatBytes(b: number | null | undefined) {
  if (b == null || b === 0) return "—";
  const units = ["B", "K", "M", "G", "T"];
  let v = b;
  let i = 0;
  while (v >= 1024 && i < units.length - 1) { v /= 1024; i++; }
  return `${v.toFixed(1)}${units[i]}`;
}

function Progress({ value, color = "bg-primary" }: { value: number; color?: string }) {
  const pct = Math.max(0, Math.min(100, value));
  return (
    <div className="h-2 w-full rounded bg-muted overflow-hidden">
      <div className={`h-full ${color}`} style={{ width: `${pct}%` }} />
    </div>
  );
}

export function System() {
  const [info, setInfo] = React.useState<any>(null);
  const [metrics, setMetrics] = React.useState<any>(null);
  const [upgrade, setUpgrade] = React.useState<any>(null);
  const [events, setEvents] = React.useState<any>(null);
  const [error, setError] = React.useState<string | null>(null);
  const [live, setLive] = React.useState(true);
  const [showRaw, setShowRaw] = React.useState(false);
  const [showAllDisks, setShowAllDisks] = React.useState(false);

  const loadInfo = React.useCallback(() => {
    apiFetch("/system/info").then(setInfo).catch((e: any) => setError(String(e.message || e)));
    apiFetch("/system/upgrade").then(setUpgrade).catch(() => {});
    apiFetch("/system/events").then(setEvents).catch(() => {});
  }, []);
  const loadMetrics = React.useCallback(() => {
    apiFetch("/system/metrics").then((d: any) => {
      setMetrics(d);
    }).catch(() => {});
  }, []);

  React.useEffect(() => { loadInfo(); }, [loadInfo]);
  React.useEffect(() => {
    loadMetrics();
    if (!live) return;
    const id = setInterval(loadMetrics, 5000);
    return () => clearInterval(id);
  }, [live, loadMetrics]);

  const cpuPerCoreData = React.useMemo(() => {
    const arr: number[] = metrics?.cpu_per_core || info?.cpu?.per_core || [];
    return arr.map((v: number, i: number) => ({ core: `core ${i}`, usage: v }));
  }, [metrics, info]);

  const memory = metrics?.memory || info?.memory;
  const disks: any[] = metrics?.disks || info?.disk || [];
  // Filter disks: hide loops/squashfs unless toggle
  const filteredDisks = React.useMemo(() => {
    if (showAllDisks) return disks;
    return disks.filter((d: any) => {
      if (d.device?.startsWith("/dev/loop")) return false;
      if (d.fstype === "squashfs") return false;
      if (d.mount?.startsWith("/snap")) return false;
      if (d.mount === "/dev" || d.mount === "/run") return false;
      return true;
    });
  }, [disks, showAllDisks]);

  const memoryPie = React.useMemo(() => {
    if (!memory) return [];
    const total = memory.total_bytes || memory.total || 0;
    const avail = memory.available_bytes ?? memory.available ?? 0;
    const used = memory.used_bytes ?? memory.used ?? (total - avail);
    const free = memory.free_bytes ?? memory.free ?? avail;
    if (!total) return [];
    return [
      { name: "Used", value: used },
      { name: "Available", value: avail || free },
    ];
  }, [memory]);

  if (error) {
    return <div className="p-4 text-sm text-destructive">Failed to load system info: {error}</div>;
  }
  if (!info && !metrics) {
    return <div className="p-4 text-sm text-muted-foreground">Loading system info…</div>;
  }

  const os = info?.os || {};
  // Contract: backend returns load_avg as [1m,5m,15m] triple; tolerate legacy
  // bare-number payloads (and null) so one shape skew never crashes the route.
  const raw: unknown = metrics?.load_avg ?? info?.load_avg ?? null;
  const loadAvg: number[] | null = Array.isArray(raw)
    ? (raw as number[]).filter((v): v is number => typeof v === "number")
    : typeof raw === "number"
      ? [raw]
      : null;
  const cpuTotal: number | null = metrics?.cpu_total ?? info?.cpu?.total_percent ?? null;
  const cpuCores: number = metrics?.cpu_cores ?? info?.cpu?.cores ?? 0;
  const uptime: string = metrics?.uptime ?? info?.uptime ?? "—";
  const hostname: string = metrics?.hostname ?? info?.hostname ?? "—";

  return (
    <div className="p-4 space-y-4">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <h1 className="text-2xl font-bold">System Status</h1>
        <div className="flex items-center gap-2">
          <button onClick={() => { loadInfo(); loadMetrics(); }} className="rounded-md border bg-background px-3 py-1.5 text-sm hover:bg-muted">Refresh</button>
          <label className="flex items-center gap-1.5 text-sm">
            <input type="checkbox" checked={live} onChange={(e) => setLive(e.target.checked)} />
            Live (5s)
          </label>
        </div>
      </div>

      {/* OS + Uptime + Load */}
      <div className="grid gap-4 md:grid-cols-3">
        <Card>
          <CardHeader><CardTitle className="text-sm">OS Information</CardTitle></CardHeader>
          <CardContent className="space-y-1 text-sm">
            <div><span className="text-muted-foreground">OS:</span> {os.name || "—"}</div>
            <div><span className="text-muted-foreground">Version:</span> {os.version || "—"}</div>
            <div className="break-all"><span className="text-muted-foreground">Kernel:</span> {os.kernel || "—"}</div>
            <div><span className="text-muted-foreground">Hostname:</span> {hostname}</div>
            <div><span className="text-muted-foreground">Date:</span> {info?.date || "—"}</div>
            <div><span className="text-muted-foreground">Type:</span> {os.type || info?.platform || "—"}</div>
            <div><span className="text-muted-foreground">Python:</span> {info?.python_version || "—"}</div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader><CardTitle className="text-sm">Uptime & Load</CardTitle></CardHeader>
          <CardContent className="space-y-2 text-sm">
            <div><span className="text-muted-foreground">Uptime:</span> <span className="font-mono text-xs">{uptime}</span></div>
            <div>
              <div className="text-muted-foreground mb-1">Load average (1m, 5m, 15m)</div>
              <div className="flex gap-2">
                {loadAvg ? loadAvg.map((v: number, i: number) => (
                  <span key={i} className="rounded bg-muted px-2 py-1 font-mono text-xs">{v.toFixed(2)}</span>
                )) : <span className="text-muted-foreground">—</span>}
              </div>
              <div className="text-xs text-muted-foreground mt-1">Cores: {cpuCores} · {loadAvg && cpuCores ? `1m per core ${(loadAvg[0]/Math.max(1,cpuCores)).toFixed(2)}` : ""}</div>
            </div>
            <div className="pt-2">
              <div className="flex justify-between text-xs"><span>CPU total</span><span>{cpuTotal != null ? `${cpuTotal}%` : "—"}</span></div>
              <Progress value={cpuTotal ?? 0} color={ (cpuTotal ?? 0) > 80 ? "bg-red-500" : (cpuTotal ?? 0) > 60 ? "bg-amber-500" : "bg-emerald-500"} />
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader><CardTitle className="text-sm">Memory</CardTitle></CardHeader>
          <CardContent className="space-y-2">
            {memory ? (
              <>
                <div className="flex justify-between text-sm"><span>Total</span><span className="font-mono">{formatBytes(memory.total_bytes ?? memory.total)}</span></div>
                <div className="flex justify-between text-sm"><span>Used</span><span className="font-mono">{formatBytes(memory.used_bytes ?? memory.used)}</span></div>
                <div className="flex justify-between text-sm"><span>Available</span><span className="font-mono">{formatBytes(memory.available_bytes ?? memory.available ?? memory.free_bytes)}</span></div>
                <div className="flex justify-between text-xs text-muted-foreground"><span>Free</span><span className="font-mono">{formatBytes(memory.free_bytes ?? memory.free)}</span></div>
                <Progress value={memory.percent ?? 0} color={(memory.percent ?? 0) > 85 ? "bg-red-500" : (memory.percent ?? 0) > 70 ? "bg-amber-500" : "bg-primary"} />
                <div className="text-xs text-center text-muted-foreground">{memory.percent ?? 0}% used</div>
                <button onClick={() => setShowRaw(v => !v)} className="text-xs text-primary hover:underline w-full mt-1">{showRaw ? "Hide raw" : "Show raw"}</button>
                {showRaw && <pre className="text-xs bg-muted p-2 rounded overflow-auto max-h-32 whitespace-pre-wrap">{memory.raw || "—"}</pre>}
              </>
            ) : <div className="text-sm text-muted-foreground">No memory data</div>}
          </CardContent>
        </Card>
      </div>

      {/* Upgrade + Events (Rust: /system/upgrade, /system/events) */}
      {(upgrade || events) && (
        <div className="grid gap-4 md:grid-cols-2">
          {upgrade && (
            <Card>
              <CardHeader><CardTitle className="text-sm">Upgrade</CardTitle></CardHeader>
              <CardContent className="space-y-1 text-sm">
                <div><span className="text-muted-foreground">Boot migrate:</span> {String(upgrade.boot_migrate ?? "—")}</div>
                <div><span className="text-muted-foreground">Pending:</span> {String(upgrade.pending ?? "—")}</div>
                <div><span className="text-muted-foreground">Version:</span> {upgrade.version || "—"}</div>
                {upgrade.note && <div className="text-xs text-muted-foreground">{upgrade.note}</div>}
              </CardContent>
            </Card>
          )}
          {events && (
            <Card>
              <CardHeader><CardTitle className="text-sm">Events (event socket)</CardTitle></CardHeader>
              <CardContent className="space-y-1 text-sm">
                <div><span className="text-muted-foreground">Transport:</span> {events.transport || "—"}</div>
                <div><span className="text-muted-foreground">FreeSWITCH connected:</span> {String(events.freeswitch?.connected ?? events.freeswitch ?? "—")}</div>
                <div><span className="text-muted-foreground">Event guard logs:</span> {events.event_guard_logs ?? "—"}</div>
              </CardContent>
            </Card>
          )}
        </div>
      )}

      {/* CPU per core + Memory pie */}
      <div className="grid gap-4 md:grid-cols-2">
        <Card>
          <CardHeader><CardTitle>CPU per Core (%)</CardTitle></CardHeader>
          <CardContent>
            <div className="h-56">
              {cpuPerCoreData.length ? (
                <ResponsiveContainer width="100%" height="100%">
                  <BarChart data={cpuPerCoreData}>
                    <CartesianGrid strokeDasharray="3 3" />
                    <XAxis dataKey="core" tick={{ fontSize: 10 }} interval={0} angle={-20} textAnchor="end" height={50} />
                    <YAxis domain={[0, 100]} tick={{ fontSize: 12 }} />
                    <Tooltip />
                    <Bar dataKey="usage" fill="hsl(var(--primary))" radius={[4,4,0,0]} />
                  </BarChart>
                </ResponsiveContainer>
              ) : (
                <div className="flex h-full items-center justify-center text-sm text-muted-foreground">No per-core data — polling…</div>
              )}
            </div>
            <p className="text-xs text-center text-muted-foreground mt-1">Live every 5s · {cpuCores} cores · total {cpuTotal ?? "—"}%</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader><CardTitle>Memory Usage</CardTitle></CardHeader>
          <CardContent>
            <div className="h-56">
              {memoryPie.length && memoryPie.some(d => d.value > 0) ? (
                <ResponsiveContainer width="100%" height="100%">
                  <PieChart>
                    <Pie data={memoryPie} dataKey="value" nameKey="name" cx="50%" cy="50%" innerRadius={50} outerRadius={80} label={({ name, percent }) => `${name} ${((percent ?? 0)*100).toFixed(0)}%`}>
                      <Cell fill="#3b82f6" />
                      <Cell fill="#10b981" />
                    </Pie>
                    <Tooltip formatter={(v: any) => formatBytes(v)} />
                    <Legend />
                  </PieChart>
                </ResponsiveContainer>
              ) : <div className="flex h-full items-center justify-center text-sm text-muted-foreground">No memory breakdown</div>}
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Disk */}
      <Card>
        <CardHeader className="flex flex-row items-center justify-between space-y-0">
          <CardTitle>Disk Usage</CardTitle>
          <label className="flex items-center gap-1.5 text-xs font-normal">
            <input type="checkbox" checked={showAllDisks} onChange={e => setShowAllDisks(e.target.checked)} />
            Show all (incl. snaps)
          </label>
        </CardHeader>
        <CardContent>
          <div className="overflow-auto">
            <table className="w-full text-sm">
              <thead className="text-xs text-muted-foreground">
                <tr className="border-b">
                  <th className="text-left p-2">Device</th>
                  <th className="text-left p-2">Mount</th>
                  <th className="text-left p-2">Type</th>
                  <th className="text-right p-2">Size</th>
                  <th className="text-right p-2">Used</th>
                  <th className="text-right p-2">Avail</th>
                  <th className="text-left p-2 w-32">Use%</th>
                </tr>
              </thead>
              <tbody>
                {filteredDisks.length ? filteredDisks.map((d: any, i: number) => (
                  <tr key={i} className="border-b last:border-0 hover:bg-muted/50">
                    <td className="p-2 font-mono text-xs">{d.device}</td>
                    <td className="p-2 font-mono text-xs">{d.mount}</td>
                    <td className="p-2 text-xs">{d.fstype || "—"}</td>
                    <td className="p-2 text-right font-mono text-xs">{d.size || formatBytes(d.size_bytes)}</td>
                    <td className="p-2 text-right font-mono text-xs">{d.used || formatBytes(d.used_bytes)}</td>
                    <td className="p-2 text-right font-mono text-xs">{d.avail || formatBytes(d.avail_bytes)}</td>
                    <td className="p-2">
                      <div className="flex items-center gap-2">
                        <div className="flex-1"><Progress value={d.use_percent ?? 0} color={(d.use_percent ?? 0) > 85 ? "bg-red-500" : (d.use_percent ?? 0) > 75 ? "bg-amber-500" : "bg-primary"} /></div>
                        <span className="text-xs w-10 text-right">{d.use_percent ?? 0}%</span>
                      </div>
                    </td>
                  </tr>
                )) : (
                  <tr><td colSpan={7} className="p-4 text-center text-muted-foreground">No disk data</td></tr>
                )}
              </tbody>
            </table>
          </div>
          {disks.length > filteredDisks.length && !showAllDisks && (
            <div className="text-xs text-center text-muted-foreground mt-2">{disks.length - filteredDisks.length} snap/loop devices hidden</div>
          )}
        </CardContent>
      </Card>

      {/* Raw info dump */}
      <Card>
        <CardHeader><CardTitle className="text-sm">Raw System Info (debug)</CardTitle></CardHeader>
        <CardContent>
          <pre className="text-xs bg-muted p-3 rounded overflow-auto max-h-64">{JSON.stringify(info, null, 2)}</pre>
        </CardContent>
      </Card>
    </div>
  );
}
