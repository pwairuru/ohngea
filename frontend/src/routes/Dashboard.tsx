import * as React from "react";
import { useNavigate } from "react-router-dom";
import { Card, CardContent, CardHeader, CardTitle } from "../components/ui/card";
import { apiFetch } from "../lib/api";
import {
  LineChart,
  Line,
  BarChart,
  Bar,
  PieChart,
  Pie,
  Cell,
  XAxis,
  YAxis,
  Tooltip,
  ResponsiveContainer,
  Legend,
  CartesianGrid,
  AreaChart,
  Area,
} from "recharts";

const COLORS = ["hsl(var(--primary))", "#3b82f6", "#10b981", "#f59e0b", "#ef4444", "#8b5cf6", "#06b6d4", "#ec4899"];
const PIE_COLORS = ["#3b82f6", "#10b981", "#f59e0b", "#ef4444", "#8b5cf6", "#06b6d4", "#eab308", "#ec4899"];

export function Dashboard() {
  const navigate = useNavigate();
  const [stats, setStats] = React.useState<any>({ total_calls: 0, today_calls: 0, avg_duration: 0 });
  const [daily, setDaily] = React.useState<any[]>([]);
  const [byDirection, setByDirection] = React.useState<any[]>([]);
  const [byHangup, setByHangup] = React.useState<any[]>([]);
  const [topExt, setTopExt] = React.useState<any[]>([]);
  const [hourly, setHourly] = React.useState<any[]>([]);
  const [systemCounts, setSystemCounts] = React.useState<any>({});
  const [answerRate, setAnswerRate] = React.useState<any>({ answered: 0, missed: 0, answer_rate: 0 });
  const [sipStatus, setSipStatus] = React.useState<any>(null);
  const [sysMetrics, setSysMetrics] = React.useState<any>(null);
  const [liveActiveCalls, setLiveActiveCalls] = React.useState<number | null>(null);
  const [liveRegs, setLiveRegs] = React.useState<number | null>(null);
  const [liveUpdated, setLiveUpdated] = React.useState<Date | null>(null);

  React.useEffect(() => {
    apiFetch("/analytics/summary").then(setStats).catch(() => {});
    apiFetch("/analytics/daily?days=7").then((d) => setDaily(d.data || [])).catch(() => {});
    apiFetch("/analytics/by_direction").then((d) => setByDirection(d.data || [])).catch(() => {});
    apiFetch("/analytics/by_hangup").then((d) => setByHangup(d.data || [])).catch(() => {});
    apiFetch("/analytics/top_extensions?limit=5").then((d) => setTopExt(d.data || [])).catch(() => {});
    apiFetch("/analytics/hourly").then((d) => {
      const raw = d.data || [];
      const mapped = raw.map((r: any) => {
        try {
          const dt = new Date(r.hour);
          const label = isNaN(dt.getTime()) ? r.hour : dt.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
          return { hour: label, count: r.count, raw: r.hour };
        } catch {
          return { hour: r.hour, count: r.count };
        }
      });
      setHourly(mapped);
    }).catch(() => {});
    apiFetch("/analytics/system_counts").then(setSystemCounts).catch(() => {});
    apiFetch("/analytics/answer_rate").then(setAnswerRate).catch(() => {});
    apiFetch("/sip_profiles/status").then(setSipStatus).catch(() => {});
    apiFetch("/system/metrics").then(setSysMetrics).catch(() => {});
  }, []);

  // Live system metrics polling (CPU/RAM/Disk) — 5s
  React.useEffect(() => {
    const loadSys = () => apiFetch("/system/metrics").then(setSysMetrics).catch(() => {});
    const id = window.setInterval(() => { if (!document.hidden) loadSys(); }, 5000);
    const onVis = () => { if (!document.hidden) loadSys(); };
    document.addEventListener("visibilitychange", onVis);
    return () => { window.clearInterval(id); document.removeEventListener("visibilitychange", onVis); };
  }, []);

  // Live FreeSWITCH polling for Active Calls + Registrations (important numbers)
  React.useEffect(() => {
    const loadLive = async () => {
      try {
        const r: any = await apiFetch("/active_calls?per_page=1");
        setLiveActiveCalls(r.total ?? r.data?.length ?? 0);
      } catch {
        setLiveActiveCalls(null);
      }
      try {
        const r: any = await apiFetch("/registrations?per_page=1");
        setLiveRegs(r.total ?? r.data?.length ?? 0);
      } catch {
        setLiveRegs(null);
      }
      setLiveUpdated(new Date());
    };
    loadLive();
    const id = window.setInterval(() => {
      if (document.hidden) return;
      loadLive();
    }, 10000);
    const onVis = () => {
      if (!document.hidden) loadLive();
    };
    document.addEventListener("visibilitychange", onVis);
    return () => {
      window.clearInterval(id);
      document.removeEventListener("visibilitychange", onVis);
    };
  }, []);

  const directionData = byDirection.length ? byDirection : [];
  const hangupData = byHangup.length ? byHangup : [];
  const topData = topExt.length ? topExt : [];
  const hourlyData = hourly.length ? hourly : [];
  const answerData = answerRate.total ? [
    { name: "Answered", value: answerRate.answered },
    { name: "Missed", value: answerRate.missed },
  ] : [];
  const extensionStatusData = systemCounts.extensions != null ? [
    { name: "Enabled", value: systemCounts.enabled_extensions ?? 0 },
    { name: "Disabled", value: systemCounts.disabled_extensions ?? 0 },
  ].filter(d => d.value > 0) : [];
  const systemOverviewData = systemCounts.extensions != null ? [
    { name: "Ext", count: systemCounts.extensions || 0 },
    { name: "Devices", count: systemCounts.devices || 0 },
    { name: "Gateways", count: systemCounts.gateways || 0 },
    { name: "SIP Profiles", count: systemCounts.sip_profiles || 0 },
    { name: "Voicemails", count: systemCounts.voicemails || 0 },
  ] : [];
  const sipProfileData = sipStatus?.profiles ? (() => {
    const running = sipStatus.profiles.filter((p: any) => p.running).length;
    const notRunning = sipStatus.profiles.length - running;
    if (sipStatus.connected === false) return [{ name: "Unknown", value: sipStatus.profiles.length }];
    return [
      { name: "Running", value: running },
      { name: "Stopped", value: notRunning },
    ].filter(d => d.value > 0);
  })() : [];

  return (
    <div className="p-4 space-y-4">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <h1 className="text-2xl font-bold">Dashboard</h1>
        <div className="flex items-center gap-2">
          <button
            data-testid="dashboard-add-extension"
            onClick={() => navigate("/extensions")}
            className="inline-flex items-center gap-1.5 rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 shrink-0"
          >
            + Add Extension
          </button>
          <button
            onClick={() => navigate("/devices")}
            className="hidden sm:inline-flex rounded-md border bg-background px-3 py-2 text-sm hover:bg-muted"
          >
            Add Device
          </button>
          <button
            onClick={() => navigate("/gateways")}
            className="hidden lg:inline-flex rounded-md border bg-background px-3 py-2 text-sm hover:bg-muted"
          >
            Add Gateway
          </button>
        </div>
      </div>

      {/* Primary KPI row */}
      <div className="grid gap-4 md:grid-cols-4">
        <Card><CardHeader><CardTitle>Today Calls</CardTitle></CardHeader><CardContent><div className="text-3xl font-bold">{stats.today_calls ?? 0}</div><p className="text-xs text-muted-foreground">calls today · {Math.round(stats.avg_duration || 0)}s avg</p></CardContent></Card>
        <Card><CardHeader><CardTitle>Total Calls</CardTitle></CardHeader><CardContent><div className="text-3xl font-bold">{stats.total_calls ?? 0}</div><p className="text-xs text-muted-foreground">all time</p></CardContent></Card>
        <Card><CardHeader><CardTitle>Avg Duration</CardTitle></CardHeader><CardContent><div className="text-3xl font-bold">{Math.round(stats.avg_duration || 0)}s</div><p className="text-xs text-muted-foreground">avg · {Math.round(answerRate.avg_billsec || 0)}s billsec</p></CardContent></Card>
        <Card><CardHeader><CardTitle>Answer Rate</CardTitle></CardHeader><CardContent><div className="text-3xl font-bold">{answerRate.answer_rate != null ? `${Math.round(answerRate.answer_rate)}%` : "—"}</div><p className="text-xs text-muted-foreground">{answerRate.answered ?? 0} answered · {answerRate.missed ?? 0} missed</p></CardContent></Card>
      </div>

      {/* Live FreeSWITCH counts — important numbers, polled actively */}
      <div className="grid gap-4 md:grid-cols-2">
        <Card className="cursor-pointer hover:bg-muted/50 border-l-4 border-l-green-500" onClick={() => navigate("/active_calls")} data-testid="dashboard-active-calls">
          <CardHeader className="pb-2"><CardTitle className="text-sm flex items-center justify-between">Active Calls <span className={`h-2 w-2 rounded-full ${liveActiveCalls != null ? "bg-green-500 animate-pulse" : "bg-gray-300"}`} title={liveActiveCalls != null ? "Live polling" : "FS unavailable"} /></CardTitle></CardHeader>
          <CardContent>
            <div className="text-3xl font-bold" data-testid="dashboard-active-calls-count">{liveActiveCalls != null ? liveActiveCalls : "—"}</div>
            <p className="text-xs text-muted-foreground">live from FreeSWITCH · `show channels` {liveUpdated ? `· ${liveUpdated.toLocaleTimeString()}` : ""} · polling 10s</p>
          </CardContent>
        </Card>
        <Card className="cursor-pointer hover:bg-muted/50 border-l-4 border-l-blue-500" onClick={() => navigate("/registrations")} data-testid="dashboard-registrations">
          <CardHeader className="pb-2"><CardTitle className="text-sm flex items-center justify-between">Registrations <span className={`h-2 w-2 rounded-full ${liveRegs != null ? "bg-blue-500 animate-pulse" : "bg-gray-300"}`} title={liveRegs != null ? "Live polling" : "FS unavailable"} /></CardTitle></CardHeader>
          <CardContent>
            <div className="text-3xl font-bold" data-testid="dashboard-registrations-count">
              {liveRegs != null ? liveRegs : "—"}
              {liveRegs != null && systemCounts.enabled_extensions != null ? <span className="text-lg font-normal text-muted-foreground"> / {systemCounts.enabled_extensions}</span> : ""}
            </div>
            <p className="text-xs text-muted-foreground">{liveRegs != null ? `${liveRegs} active` : "live"} · {systemCounts.extensions != null ? `${systemCounts.extensions} total extensions` : "loading…"} {liveUpdated ? `· ${liveUpdated.toLocaleTimeString()}` : ""} · sofia reg</p>
          </CardContent>
        </Card>
      </div>

      {/* System health — host metrics (CPU/RAM/Disk) — 5s live */}
      <div className="grid gap-4 md:grid-cols-4">
        <Card className="cursor-pointer hover:bg-muted/50 border-l-4 border-l-amber-500" onClick={() => navigate("/system")} data-testid="dashboard-cpu">
          <CardHeader className="pb-2"><CardTitle className="text-sm flex items-center justify-between">CPU <span className={`h-2 w-2 rounded-full ${sysMetrics ? "bg-amber-500 animate-pulse" : "bg-gray-300"}`} /></CardTitle></CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{sysMetrics?.cpu_total != null ? `${sysMetrics.cpu_total}%` : "—"} <span className="text-sm font-normal text-muted-foreground">{sysMetrics?.cpu_cores ? `${sysMetrics.cpu_cores} cores` : ""}</span></div>
            <div className="mt-1 h-1.5 w-full rounded bg-muted overflow-hidden">
              <div className="h-full bg-amber-500 transition-all" style={{ width: `${Math.min(100, sysMetrics?.cpu_total ?? 0)}%` }} />
            </div>
            <p className="text-xs text-muted-foreground mt-1 truncate">
              {sysMetrics?.cpu_per_core?.length ? `per-core: ${sysMetrics.cpu_per_core.map((v:any)=>Math.round(v)).join("% ") + "%"}`
                : sysMetrics?.load_avg ? `load ${sysMetrics.load_avg.map((v:any)=>v.toFixed(2)).join(" ")}` : "polling /system/metrics 5s"}
            </p>
          </CardContent>
        </Card>
        <Card className="cursor-pointer hover:bg-muted/50 border-l-4 border-l-sky-500" onClick={() => navigate("/system")} data-testid="dashboard-memory">
          <CardHeader className="pb-2"><CardTitle className="text-sm flex items-center justify-between">Memory <span className={`h-2 w-2 rounded-full ${sysMetrics?.memory ? "bg-sky-500 animate-pulse" : "bg-gray-300"}`} /></CardTitle></CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{sysMetrics?.memory?.percent != null ? `${Math.round(sysMetrics.memory.percent)}%` : "—"} <span className="text-sm font-normal text-muted-foreground">used</span></div>
            <div className="mt-1 h-1.5 w-full rounded bg-muted overflow-hidden">
              <div className="h-full bg-sky-500 transition-all" style={{ width: `${Math.min(100, sysMetrics?.memory?.percent ?? 0)}%` }} />
            </div>
            <p className="text-xs text-muted-foreground mt-1 truncate">
              {sysMetrics?.memory ? `${((sysMetrics.memory.used ?? 0)/1024/1024/1024).toFixed(1)}G / ${((sysMetrics.memory.total ?? 0)/1024/1024/1024).toFixed(1)}G`
                : "polling…" }
            </p>
          </CardContent>
        </Card>
        <Card className="cursor-pointer hover:bg-muted/50 border-l-4 border-l-violet-500" onClick={() => navigate("/system")} data-testid="dashboard-disk">
          <CardHeader className="pb-2"><CardTitle className="text-sm flex items-center justify-between">Disk <span className={`h-2 w-2 rounded-full ${sysMetrics?.disk_percent != null ? "bg-violet-500 animate-pulse" : "bg-gray-300"}`} /></CardTitle></CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{sysMetrics?.disk_percent != null ? `${Math.round(sysMetrics.disk_percent)}%` : "—"} <span className="text-sm font-normal text-muted-foreground">used</span></div>
            <div className="mt-1 h-1.5 w-full rounded bg-muted overflow-hidden">
              <div className="h-full bg-violet-500 transition-all" style={{ width: `${Math.min(100, sysMetrics?.disk_percent ?? 0)}%` }} />
            </div>
            <p className="text-xs text-muted-foreground mt-1 truncate">
              {sysMetrics?.disks?.[0] ? `${sysMetrics.disks[0].mount} ${sysMetrics.disks[0].size || ""}` : sysMetrics?.uptime ? sysMetrics.uptime.slice(0,40) : "polling…"}
            </p>
          </CardContent>
        </Card>
        <Card className="cursor-pointer hover:bg-muted/50 border-l-4 border-l-emerald-500" onClick={() => navigate("/system")} data-testid="dashboard-system">
          <CardHeader className="pb-2"><CardTitle className="text-sm flex items-center justify-between">System <span className="text-[10px] text-muted-foreground">→ details</span></CardTitle></CardHeader>
          <CardContent>
            <div className="text-sm font-mono truncate">{sysMetrics?.hostname || "—"}</div>
            <p className="text-xs text-muted-foreground truncate">{sysMetrics?.load_avg ? `load ${sysMetrics.load_avg.join(" ")}` : "—"}</p>
            <p className="text-xs text-muted-foreground truncate mt-1" title={sysMetrics?.uptime || ""}>{sysMetrics?.uptime ? sysMetrics.uptime.replace(/^.*up\s+/, "up ").slice(0,45) : "—"}</p>
          </CardContent>
        </Card>
      </div>

      {/* System counts row */}
      <div className="grid gap-4 md:grid-cols-4">
        <Card className="cursor-pointer hover:bg-muted/50" onClick={() => navigate("/extensions")}><CardHeader><CardTitle className="text-sm">Extensions</CardTitle></CardHeader><CardContent><div className="text-2xl font-bold">{systemCounts.extensions ?? "—"}</div><p className="text-xs text-muted-foreground">{systemCounts.enabled_extensions ?? 0} enabled · {systemCounts.disabled_extensions ?? 0} disabled</p></CardContent></Card>
        <Card className="cursor-pointer hover:bg-muted/50" onClick={() => navigate("/devices")}><CardHeader><CardTitle className="text-sm">Devices</CardTitle></CardHeader><CardContent><div className="text-2xl font-bold">{systemCounts.devices ?? "—"}</div><p className="text-xs text-muted-foreground">provisioned</p></CardContent></Card>
        <Card className="cursor-pointer hover:bg-muted/50" onClick={() => navigate("/gateways")}><CardHeader><CardTitle className="text-sm">Gateways</CardTitle></CardHeader><CardContent><div className="text-2xl font-bold">{systemCounts.gateways ?? "—"}</div><p className="text-xs text-muted-foreground">SIP trunks</p></CardContent></Card>
        <Card className="cursor-pointer hover:bg-muted/50" onClick={() => navigate("/sip_profiles")}><CardHeader><CardTitle className="text-sm">SIP Profiles</CardTitle></CardHeader><CardContent><div className="text-2xl font-bold">{systemCounts.sip_profiles ?? "—"}</div><p className="text-xs text-muted-foreground">{sipStatus ? `${sipStatus.profiles?.filter((p:any)=>p.running).length ?? 0} running` : "status"}</p></CardContent></Card>
      </div>

      {/* Main charts - 7 day and hourly */}
      <div className="grid gap-4 md:grid-cols-2">
        <Card>
          <CardHeader><CardTitle>CDR Daily (7 days)</CardTitle></CardHeader>
          <CardContent>
            <div className="h-48">
              {daily.length ? (
                <ResponsiveContainer width="100%" height="100%">
                  <AreaChart data={daily}>
                    <CartesianGrid strokeDasharray="3 3" />
                    <XAxis dataKey="day" tick={{ fontSize: 12 }} />
                    <YAxis tick={{ fontSize: 12 }} />
                    <Tooltip />
                    <Area type="monotone" dataKey="count" stroke="hsl(var(--primary))" fill="hsl(var(--primary))" fillOpacity={0.2} />
                  </AreaChart>
                </ResponsiveContainer>
              ) : (
                <div className="flex h-full items-center justify-center text-sm text-muted-foreground">No CDR data yet — start generating calls</div>
              )}
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader><CardTitle>Hourly Today</CardTitle></CardHeader>
          <CardContent>
            <div className="h-48">
              {hourlyData.length ? (
                <ResponsiveContainer width="100%" height="100%">
                  <BarChart data={hourlyData}>
                    <CartesianGrid strokeDasharray="3 3" />
                    <XAxis dataKey="hour" tick={{ fontSize: 10 }} interval="preserveStartEnd" />
                    <YAxis tick={{ fontSize: 12 }} />
                    <Tooltip />
                    <Bar dataKey="count" fill="hsl(var(--primary))" radius={[4, 4, 0, 0]} />
                  </BarChart>
                </ResponsiveContainer>
              ) : (
                <div className="flex h-full items-center justify-center text-sm text-muted-foreground">No hourly data</div>
              )}
            </div>
          </CardContent>
        </Card>
      </div>

      <div className="grid gap-4 md:grid-cols-2">
        <Card>
          <CardHeader><CardTitle>Calls by Direction</CardTitle></CardHeader>
          <CardContent>
            <div className="h-56">
              {directionData.length ? (
                <ResponsiveContainer width="100%" height="100%">
                  <PieChart>
                    <Pie data={directionData} dataKey="count" nameKey="direction" cx="50%" cy="50%" outerRadius={70} label={({ name, percent }) => `${name} ${((percent ?? 0) * 100).toFixed(0)}%`}>
                      {directionData.map((_: any, i: number) => <Cell key={i} fill={PIE_COLORS[i % PIE_COLORS.length]} />)}
                    </Pie>
                    <Tooltip />
                    <Legend />
                  </PieChart>
                </ResponsiveContainer>
              ) : (
                <div className="flex h-full items-center justify-center text-sm text-muted-foreground">No direction breakdown</div>
              )}
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader><CardTitle>Answered vs Missed</CardTitle></CardHeader>
          <CardContent>
            <div className="h-56">
              {answerData.length && answerData.some(d => d.value > 0) ? (
                <ResponsiveContainer width="100%" height="100%">
                  <PieChart>
                    <Pie data={answerData} dataKey="value" nameKey="name" cx="50%" cy="50%" innerRadius={45} outerRadius={70} label>
                      <Cell fill="#10b981" />
                      <Cell fill="#ef4444" />
                    </Pie>
                    <Tooltip />
                    <Legend />
                  </PieChart>
                </ResponsiveContainer>
              ) : (
                <div className="flex h-full items-center justify-center text-sm text-muted-foreground">No call status data</div>
              )}
            </div>
            <p className="text-xs text-center text-muted-foreground mt-1">Billsec &gt; 0 = answered</p>
          </CardContent>
        </Card>
      </div>

      <div className="grid gap-4 md:grid-cols-2">
        <Card>
          <CardHeader><CardTitle>Top Extensions</CardTitle></CardHeader>
          <CardContent>
            <div className="h-56">
              {topData.length ? (
                <ResponsiveContainer width="100%" height="100%">
                  <BarChart data={topData} layout="vertical">
                    <CartesianGrid strokeDasharray="3 3" />
                    <XAxis type="number" tick={{ fontSize: 12 }} />
                    <YAxis dataKey="extension" type="category" tick={{ fontSize: 12 }} width={70} />
                    <Tooltip />
                    <Bar dataKey="count" fill="#3b82f6" radius={[0, 4, 4, 0]} />
                  </BarChart>
                </ResponsiveContainer>
              ) : (
                <div className="flex h-full items-center justify-center text-sm text-muted-foreground">No extension stats</div>
              )}
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader><CardTitle>Hangup Causes</CardTitle></CardHeader>
          <CardContent>
            <div className="h-56">
              {hangupData.length ? (
                <ResponsiveContainer width="100%" height="100%">
                  <BarChart data={hangupData}>
                    <CartesianGrid strokeDasharray="3 3" />
                    <XAxis dataKey="hangup_cause" tick={{ fontSize: 10 }} interval={0} angle={-15} textAnchor="end" height={60} />
                    <YAxis tick={{ fontSize: 12 }} />
                    <Tooltip />
                    <Bar dataKey="count" fill="#8b5cf6" radius={[4, 4, 0, 0]} />
                  </BarChart>
                </ResponsiveContainer>
              ) : (
                <div className="flex h-full items-center justify-center text-sm text-muted-foreground">No hangup data</div>
              )}
            </div>
          </CardContent>
        </Card>
      </div>

      <div className="grid gap-4 md:grid-cols-2">
        <Card>
          <CardHeader><CardTitle>System Overview</CardTitle></CardHeader>
          <CardContent>
            <div className="h-56">
              {systemOverviewData.length ? (
                <ResponsiveContainer width="100%" height="100%">
                  <BarChart data={systemOverviewData}>
                    <CartesianGrid strokeDasharray="3 3" />
                    <XAxis dataKey="name" tick={{ fontSize: 11 }} />
                    <YAxis tick={{ fontSize: 12 }} />
                    <Tooltip />
                    <Bar dataKey="count" fill="hsl(var(--primary))" radius={[4, 4, 0, 0]}>
                      {systemOverviewData.map((_, i) => <Cell key={i} fill={COLORS[i % COLORS.length]} />)}
                    </Bar>
                  </BarChart>
                </ResponsiveContainer>
              ) : (
                <div className="flex h-full items-center justify-center text-sm text-muted-foreground">Loading system counts…</div>
              )}
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader><CardTitle>Extension Health</CardTitle></CardHeader>
          <CardContent>
            <div className="h-56">
              {extensionStatusData.length ? (
                <ResponsiveContainer width="100%" height="100%">
                  <PieChart>
                    <Pie data={extensionStatusData} dataKey="value" nameKey="name" cx="50%" cy="50%" innerRadius={45} outerRadius={70} label>
                      <Cell fill="#10b981" />
                      <Cell fill="#9ca3af" />
                    </Pie>
                    <Tooltip />
                    <Legend />
                  </PieChart>
                </ResponsiveContainer>
              ) : sipProfileData.length ? (
                <ResponsiveContainer width="100%" height="100%">
                  <PieChart>
                    <Pie data={sipProfileData} dataKey="value" nameKey="name" cx="50%" cy="50%" innerRadius={45} outerRadius={70} label>
                      <Cell fill="#10b981" />
                      <Cell fill="#ef4444" />
                      <Cell fill="#9ca3af" />
                    </Pie>
                    <Tooltip />
                    <Legend />
                  </PieChart>
                </ResponsiveContainer>
              ) : (
                <div className="flex h-full items-center justify-center text-sm text-muted-foreground">No extension health data</div>
              )}
            </div>
            <p className="text-xs text-center text-muted-foreground mt-1">
              {extensionStatusData.length ? "Enabled vs disabled extensions" : sipProfileData.length ? "SIP profiles running" : ""}
            </p>
          </CardContent>
        </Card>
      </div>

      <Card><CardHeader><CardTitle>Recent Activity</CardTitle></CardHeader><CardContent><p className="text-sm text-muted-foreground">No recent activity</p></CardContent></Card>
    </div>
  );
}
