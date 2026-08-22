import * as React from "react";
import { apiFetch } from "../lib/api";
import { Card, CardContent, CardHeader, CardTitle } from "../components/ui/card";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "../components/ui/table";
import { Badge } from "../components/ui/badge";
import { Button } from "../components/ui/button";
import { Label } from "../components/ui/label";
import { Dialog, DialogHeader, DialogTitle } from "../components/ui/dialog";
export function EmailQueue() {
  const [data, setData] = React.useState<any[]>([]);
  const [total, setTotal] = React.useState(0);
  const [open, setOpen] = React.useState(false);
  const [json, setJson] = React.useState(`{
  "email_from": "a@test.com",
  "email_to": "b@test.com",
  "email_subject": "test"
}`);
  const load = () => apiFetch("/email_queue").then((r:any)=>{setData(r.data||[]); setTotal(r.total??r.data?.length??0)}).catch(()=>{});
  React.useEffect(()=>{load()},[]);
  const create = async () => {
    try {
      const body = JSON.parse(json);
      await apiFetch("/email_queue", { method: "POST", body: JSON.stringify(body) });
      setOpen(false); load();
    } catch (e:any) { alert(e.message); }
  };
  return (
    <div className="p-4 space-y-4">
      <div className="flex justify-between items-center"><h1 className="text-2xl font-bold">Email Queue</h1><div className="flex gap-2"><Badge>{total} total</Badge><Button onClick={() => setOpen(true)}>Add Email Queue</Button></div></div>
      <Card><CardHeader><CardTitle>Email Queue</CardTitle></CardHeader><CardContent>
          <div className="hidden md:block"><Table><TableHeader><TableRow><TableHead>Name / ID</TableHead><TableHead>Details</TableHead></TableRow></TableHeader><TableBody>
                {data.map((d:any,i:number)=><TableRow key={i}><TableCell className="font-mono text-xs">{d["email_subject"] ?? d.name ?? JSON.stringify(d).slice(0,60)}</TableCell><TableCell className="text-xs text-muted-foreground">{JSON.stringify(d).slice(0,80)}</TableCell></TableRow>)}
                {data.length===0 && <TableRow><TableCell colSpan={2} className="text-center text-muted-foreground py-8">No email_queue — create via Add button POST /api/v1/email_queue</TableCell></TableRow>}
              </TableBody></Table></div>
          <div className="md:hidden grid gap-3">
            {data.map((d:any,i:number)=><Card key={i} className="p-3"><div className="font-mono text-xs break-all">{JSON.stringify(d).slice(0,120)}</div></Card>)}
            {data.length===0 && <div className="text-center text-muted-foreground py-4">No email_queue</div>}
          </div>
        </CardContent></Card>
      <Dialog open={open} onOpenChange={setOpen}><DialogHeader><DialogTitle>Add Email Queue</DialogTitle></DialogHeader>
        <div className="space-y-3"><div><Label>JSON payload *</Label><textarea value={json} onChange={(e)=>setJson(e.target.value)} rows={6} className="w-full rounded border p-2 font-mono text-xs" /></div>
          <div className="text-xs text-muted-foreground">Required field shown above — edit JSON and press Create.</div>
          <Button onClick={create} className="w-full">Create</Button></div></Dialog>
    </div>
  );
}
