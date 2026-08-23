import * as React from "react";
import { apiFetch } from "../lib/api";
import { Card, CardContent, CardHeader, CardTitle } from "../components/ui/card";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "../components/ui/table";
import { PageHeader } from "../components/shell/PageHeader";
import { toastError } from "../lib/toast";

export function OperatorPanel() {
  const [data, setData] = React.useState<any | null>(null);
  const load = () => apiFetch("/operator_panel").then((r:any)=> setData(r)).catch((err)=>{ toastError(err); });
  React.useEffect(()=>{load(); const id=setInterval(load, 10000); return()=>clearInterval(id);},[]);
  if (!data) return <><PageHeader title="Operator Panel" description="Operator presence and call state" /><div className="p-4 text-sm text-muted-foreground">Loading…</div></>;
  const active = data.active || [];
  const extTotal = data.extensions_total ?? data.total ?? 0;
  return (
    <>
      <PageHeader title="Operator Panel" description="Operator presence and call state" />
      <div className="grid gap-4 md:grid-cols-3">
        <Card><CardHeader><CardTitle className="text-sm">Domain</CardTitle></CardHeader><CardContent><div className="font-mono text-xs break-all">{data.domain_id ?? "—"}</div><div className="text-sm mt-1">{extTotal} extensions</div></CardContent></Card>
        <Card><CardHeader><CardTitle className="text-sm">Active Channels</CardTitle></CardHeader><CardContent><div className="text-2xl font-bold">{active[0]?.count ?? active.length ?? 0}</div><div className="text-xs text-muted-foreground">{active[0]?.metric ?? "channels"}</div></CardContent></Card>
        <Card><CardHeader><CardTitle className="text-sm">Queues</CardTitle></CardHeader><CardContent><div className="text-sm">{(data.queues||[]).length ? (data.queues as any[]).map((q:any,i:number)=><div key={i} className="font-mono text-xs">{q.name ?? JSON.stringify(q).slice(0,40)}</div>) : <span className="text-muted-foreground">No queues</span>}</div></CardContent></Card>
      </div>
      <Card>
        <CardHeader><CardTitle>Active Details</CardTitle></CardHeader>
        <CardContent>
          <div className="hidden md:block">
            <Table>
              <TableHeader><TableRow><TableHead>Metric</TableHead><TableHead>Count</TableHead></TableRow></TableHeader>
              <TableBody>
                {active.length ? active.map((d:any,i:number)=><TableRow key={i}><TableCell className="font-mono text-xs">{d.metric ?? "channels"}</TableCell><TableCell>{d.count ?? 0}</TableCell></TableRow>) : <TableRow><TableCell colSpan={2} className="text-center text-muted-foreground py-8">No active channels</TableCell></TableRow>}
              </TableBody>
            </Table>
          </div>
          <div className="md:hidden grid gap-3">
            {active.length ? active.map((d:any,i:number)=><Card key={i} className="p-3"><div className="font-mono text-xs">{d.metric}: {d.count}</div></Card>) : <div className="text-center text-muted-foreground py-4">No active channels</div>}
          </div>
        </CardContent>
      </Card>
    </>
  );
}
