import * as React from "react";
import { apiFetch } from "../lib/api";
import { Card, CardContent, CardHeader, CardTitle } from "../components/ui/card";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "../components/ui/table";
import { PageHeader } from "../components/shell/PageHeader";
import { Button } from "../components/ui/button";
import { Input } from "../components/ui/input";
import { Label } from "../components/ui/label";
import { toast, toastError } from "../lib/toast";
import { useAsyncAction } from "../lib/useAsyncAction";

export function OperatorPanel() {
  const [data, setData] = React.useState<any | null>(null);
  const [calls, setCalls] = React.useState<any[]>([]);
  const [dest, setDest] = React.useState("");
  const [src, setSrc] = React.useState("");
  const [result, setResult] = React.useState("");

  const load = React.useCallback(() => {
    apiFetch("/operator_panel").then((r: any) => setData(r)).catch((err) => { toastError(err); });
    apiFetch("/operator_panel/calls").then((r: any) => setCalls(r?.data ?? [])).catch(() => setCalls([]));
  }, []);
  React.useEffect(() => { load(); const id = setInterval(load, 10000); return () => clearInterval(id); }, [load]);

  const { run: originate, pending } = useAsyncAction(async () => {
    try {
      const res: any = await apiFetch("/operator_panel/originate", {
        method: "POST",
        body: JSON.stringify({ destination: dest, source: src || undefined }),
      });
      setResult(res?.result ?? "originated");
      toast("Originated", { variant: "success" });
      load();
    } catch (err: any) { setResult(err?.message ?? "failed"); toastError(err); }
  });

  const act = async (uid: string, action: "hangup" | "eavesdrop" | "coach" | "record", ext?: string) => {
    try {
      const res: any = await apiFetch(`/operator_panel/${action}/${encodeURIComponent(uid)}`, {
        method: "POST",
        body: JSON.stringify(ext ? { extension: ext } : {}),
      });
      setResult(res?.result ?? `${action} sent`);
      toast(`${action} sent`, { variant: "success" });
      load();
    } catch (err: any) { setResult(err?.message ?? "failed"); toastError(err); }
  };

  if (!data) return <><PageHeader title="Operator Panel" description="Operator presence and call state" /><div className="p-4 text-sm text-muted-foreground">Loading…</div></>;
  const active = data.active || [];
  const extTotal = data.extensions_total ?? data.total ?? 0;

  return (
    <>
      <PageHeader title="Operator Panel" description="Operator presence, originate and call control" />
      <div className="grid gap-4 md:grid-cols-3">
        <Card><CardHeader><CardTitle className="text-sm">Domain</CardTitle></CardHeader><CardContent><div className="font-mono text-xs break-all">{data.domain_id ?? "—"}</div><div className="text-sm mt-1">{extTotal} extensions</div></CardContent></Card>
        <Card><CardHeader><CardTitle className="text-sm">Active Channels</CardTitle></CardHeader><CardContent><div className="text-2xl font-bold">{active[0]?.count ?? active.length ?? 0}</div><div className="text-xs text-muted-foreground">{active[0]?.metric ?? "channels"}</div></CardContent></Card>
        <Card><CardHeader><CardTitle className="text-sm">Queues</CardTitle></CardHeader><CardContent><div className="text-sm">{(data.queues || []).length ? (data.queues as any[]).map((q: any, i: number) => <div key={i} className="font-mono text-xs">{q.name ?? JSON.stringify(q).slice(0, 40)}</div>) : <span className="text-muted-foreground">No queues</span>}</div></CardContent></Card>
      </div>
      <Card>
        <CardHeader><CardTitle>Originate</CardTitle></CardHeader>
        <CardContent className="flex flex-wrap items-end gap-2">
          <div><Label>Source ext</Label><Input value={src} onChange={(e) => setSrc(e.target.value)} placeholder="1001" /></div>
          <div><Label>Destination</Label><Input value={dest} onChange={(e) => setDest(e.target.value)} placeholder="Destination" /></div>
          <Button onClick={originate} disabled={pending || !dest}>{pending ? "Originating…" : "Originate"}</Button>
        </CardContent>
      </Card>
      <Card>
        <CardHeader><CardTitle>Live Calls ({calls.length})</CardTitle></CardHeader>
        <CardContent>
          <div className="hidden md:block">
            <Table>
              <TableHeader><TableRow><TableHead>UUID</TableHead><TableHead>Caller</TableHead><TableHead>Destination</TableHead><TableHead className="text-right">Actions</TableHead></TableRow></TableHeader>
              <TableBody>
                {calls.length ? calls.map((c: any, i: number) => {
                  const uid = c.uuid ?? c.call_uuid ?? c.id ?? String(i);
                  return (
                    <TableRow key={String(uid)}>
                      <TableCell className="font-mono text-xs">{String(uid).slice(0, 13)}</TableCell>
                      <TableCell className="font-mono text-xs">{c.caller_id_number ?? c.caller ?? "—"}</TableCell>
                      <TableCell className="font-mono text-xs">{c.destination_number ?? c.dest ?? "—"}</TableCell>
                      <TableCell>
                        <div className="flex justify-end gap-1">
                          <Button variant="ghost" size="sm" onClick={() => act(String(uid), "eavesdrop", src || undefined)}>Eavesdrop</Button>
                          <Button variant="ghost" size="sm" onClick={() => act(String(uid), "coach", src || undefined)}>Coach</Button>
                          <Button variant="ghost" size="sm" onClick={() => act(String(uid), "record")}>Record</Button>
                          <Button variant="ghost" size="sm" className="text-destructive hover:text-destructive" onClick={() => act(String(uid), "hangup")}>Hangup</Button>
                        </div>
                      </TableCell>
                    </TableRow>
                  );
                }) : <TableRow><TableCell colSpan={4} className="text-center text-muted-foreground py-8">No live calls</TableCell></TableRow>}
              </TableBody>
            </Table>
          </div>
          <div className="md:hidden grid gap-3">
            {calls.length ? calls.map((c: any, i: number) => {
              const uid = String(c.uuid ?? c.call_uuid ?? c.id ?? i);
              return (
                <Card key={uid} className="p-3 space-y-1">
                  <div className="font-mono text-sm">{c.caller_id_number ?? "?"} → {c.destination_number ?? "?"}</div>
                  <div className="flex flex-wrap gap-1">
                    <Button variant="ghost" size="sm" onClick={() => act(uid, "eavesdrop", src || undefined)}>Eavesdrop</Button>
                    <Button variant="ghost" size="sm" onClick={() => act(uid, "coach", src || undefined)}>Coach</Button>
                    <Button variant="ghost" size="sm" onClick={() => act(uid, "record")}>Record</Button>
                    <Button variant="ghost" size="sm" className="text-destructive" onClick={() => act(uid, "hangup")}>Hangup</Button>
                  </div>
                </Card>
              );
            }) : <div className="text-center text-muted-foreground py-4">No live calls</div>}
          </div>
          {result && <pre className="text-xs bg-muted p-2 rounded overflow-auto max-h-32 mt-2">{result}</pre>}
        </CardContent>
      </Card>
    </>
  );
}
