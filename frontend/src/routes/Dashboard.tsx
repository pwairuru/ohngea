import * as React from "react";
import { Card, CardContent, CardHeader, CardTitle } from "../components/ui/card";
import { apiFetch } from "../lib/api";
import { LineChart, Line, XAxis, YAxis, Tooltip, ResponsiveContainer } from "recharts";

export function Dashboard() {
  const [stats, setStats] = React.useState<any>({ total_calls: 0, today_calls: 0, avg_duration: 0 });
  const [daily, setDaily] = React.useState<any[]>([]);

  React.useEffect(() => {
    apiFetch("/analytics/summary").then(setStats).catch(() => {});
    apiFetch("/analytics/daily?days=7").then((d) => setDaily(d.data || [])).catch(() => {});
  }, []);

  return (
    <div className="p-4 space-y-4">
      <h1 className="text-2xl font-bold">Dashboard</h1>
      <div className="grid gap-4 md:grid-cols-3">
        <Card><CardHeader><CardTitle>Active Calls</CardTitle></CardHeader><CardContent><div className="text-3xl font-bold">{stats.today_calls ?? 0}</div></CardContent></Card>
        <Card><CardHeader><CardTitle>Total Calls</CardTitle></CardHeader><CardContent><div className="text-3xl font-bold">{stats.total_calls ?? 0}</div></CardContent></Card>
        <Card><CardHeader><CardTitle>Avg Duration</CardTitle></CardHeader><CardContent><div className="text-3xl font-bold">{Math.round(stats.avg_duration || 0)}s</div></CardContent></Card>
      </div>
      <Card>
        <CardHeader><CardTitle>CDR Sparkline (7 days)</CardTitle></CardHeader>
        <CardContent>
          <div className="h-48">
            <ResponsiveContainer width="100%" height="100%">
              <LineChart data={daily.length ? daily : [{ day: "2026-08-14", count: 0 }, { day: "2026-08-15", count: 5 }]}>
                <XAxis dataKey="day" />
                <YAxis />
                <Tooltip />
                <Line type="monotone" dataKey="count" stroke="hsl(var(--primary))" />
              </LineChart>
            </ResponsiveContainer>
          </div>
        </CardContent>
      </Card>
      <div className="grid gap-4 md:grid-cols-2">
        <Card><CardHeader><CardTitle>Recent Activity</CardTitle></CardHeader><CardContent><p className="text-sm text-muted-foreground">No recent activity</p></CardContent></Card>
        <Card><CardHeader><CardTitle>Quick Add</CardTitle></CardHeader><CardContent><button className="rounded bg-primary px-4 py-2 text-primary-foreground">Add Extension</button></CardContent></Card>
      </div>
    </div>
  );
}
