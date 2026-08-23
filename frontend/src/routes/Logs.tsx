import * as React from "react";
import { apiFetch } from "../lib/api";
import { Card, CardContent, CardHeader, CardTitle } from "../components/ui/card";
import { Button } from "../components/ui/button";
import { Input } from "../components/ui/input";
import { Badge } from "../components/ui/badge";
import { RefreshCw } from "lucide-react";

export function Logs() {
  const [lines, setLines] = React.useState<string[]>([]);
  const [source, setSource] = React.useState<string>("");
  const [fs, setFs] = React.useState<any>(null);
  const [q, setQ] = React.useState("");
  const [level, setLevel] = React.useState("");
  const [auto, setAuto] = React.useState(true);

  const load = React.useCallback(async () => {
    try {
      const r: any = await apiFetch(`/logs?lines=100&q=${encodeURIComponent(q)}&level=${encodeURIComponent(level)}`);
      setLines(r.lines || []);
      setSource(r.source || "");
    } catch {}
    try {
      const r2: any = await apiFetch(`/logs/freeswitch?lines=50`);
      setFs(r2);
    } catch {}
  }, [q, level]);

  React.useEffect(() => { load(); }, [load]);
  React.useEffect(() => {
    if (!auto) return;
    const id = setInterval(load, 5000);
    return () => clearInterval(id);
  }, [auto, load]);

  const sofiaText = fs?.sofia_status || "";
  const regsText = fs?.registrations_raw || "";
  const channelsText = fs?.channels || "";
  const connected = fs?.fs_status?.connected;

  return (
    <div className="p-4 space-y-4">
      <div className="flex justify-between items-center">
        <div>
          <h1 className="text-2xl font-bold">Logs & Registrations</h1>
          <p className="text-sm text-muted-foreground">Live from FreeSWITCH via ESL — no host file, no <code>docker logs</code> needed. SIP trace is on.</p>
        </div>
        <div className="flex gap-2 items-center">
          <Badge variant={connected ? "default" : "destructive"}>{connected ? "FS connected" : connected === false ? "FS disconnected" : "checking"}</Badge>
          <Button variant="outline" size="sm" onClick={load}><RefreshCw className="h-4 w-4 mr-1" />Refresh</Button>
        </div>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        <Card>
          <CardHeader className="py-3"><CardTitle className="text-base">Sofia Status</CardTitle></CardHeader>
          <CardContent>
            <pre className="font-mono text-xs whitespace-pre-wrap bg-muted p-3 rounded max-h-[200px] overflow-auto">{sofiaText || "—"}</pre>
            {fs?.sofia_parsed && Object.keys(fs.sofia_parsed).length > 0 && (
              <div className="mt-2 flex gap-2 flex-wrap">
                {Object.entries(fs.sofia_parsed).map(([k, v]) => (
                  <Badge key={k} variant={v === "RUNNING" ? "default" : "destructive"}>{k}: {String(v)}</Badge>
                ))}
              </div>
            )}
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="py-3"><CardTitle className="text-base">Registrations / Channels</CardTitle></CardHeader>
          <CardContent>
            <pre className="font-mono text-xs whitespace-pre-wrap bg-muted p-3 rounded max-h-[200px] overflow-auto">{regsText || channelsText || "0 total — no phone registered yet. Try 1010@192.168.1.108:5060 with 1234."}</pre>
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader className="flex flex-row items-center justify-between space-y-0 py-3">
          <CardTitle className="text-base">Recent SIP / FS Logs</CardTitle>
          <span className="text-xs text-muted-foreground">source: {source || "esl"}</span>
        </CardHeader>
        <CardContent>
          <div className="flex gap-2 mb-3">
            <Input placeholder="filter e.g. 1010, REGISTER, 401" value={q} onChange={(e) => setQ(e.target.value)} className="flex-1" />
            <Input placeholder="level e.g. ERR, NOTICE" value={level} onChange={(e) => setLevel(e.target.value)} className="w-[160px]" />
            <Button variant={auto ? "default" : "outline"} size="sm" onClick={() => setAuto(!auto)}>{auto ? "Auto ✓" : "Auto"}</Button>
          </div>
          <pre className="font-mono text-xs whitespace-pre-wrap bg-black text-green-300 p-3 rounded max-h-[400px] overflow-auto">
            {lines.length ? lines.join("\n") : "No logs yet — try registering 1010@192.168.1.108:5060. Logs appear here without host SSH."}
          </pre>
          <p className="text-xs text-muted-foreground mt-2">These are from <code>GET /api/v1/logs</code> (ESL <code>sofia status</code> + <code>show channels</code>) and <code>GET /api/v1/logs/freeswitch</code>. Previously it was mock — now live.</p>
        </CardContent>
      </Card>
    </div>
  );
}
