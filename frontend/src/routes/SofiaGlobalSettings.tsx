import * as React from "react";
import { apiFetch } from "../lib/api";
import { Card, CardContent, CardHeader, CardTitle } from "../components/ui/card";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "../components/ui/table";
import { Badge } from "../components/ui/badge";
import { Button } from "../components/ui/button";
import { Label } from "../components/ui/label";
import { Dialog, DialogHeader, DialogTitle } from "../components/ui/dialog";
export function SofiaGlobalSettings() {
  const [data, setData] = React.useState<any[]>([]);
  const [total, setTotal] = React.useState(0);
  const [open, setOpen] = React.useState(false);
  const [json, setJson] = React.useState(`{
  "global_setting_name": "sip-trace",
  "global_setting_value": "false"
}`);
  const load = () => apiFetch("/sofia_global_settings").then((r:any)=>{setData(r.data||[]); setTotal(r.total??r.data?.length??0)}).catch(()=>{});
  React.useEffect(()=>{load()},[]);
  const create = async () => {
    try {
      const body = JSON.parse(json);
      await apiFetch("/sofia_global_settings", { method: "POST", body: JSON.stringify(body) });
      setOpen(false); load();
    } catch (e:any) { alert(e.message); }
  };
  return (
    <div className="p-4 space-y-4">
      <div className="flex justify-between items-center"><h1 className="text-2xl font-bold">Sofia Global Settings</h1><div className="flex gap-2"><Badge>{total} total</Badge><Button onClick={() => setOpen(true)}>Add Sofia Global Settings</Button></div></div>
      <Card><CardHeader><CardTitle>Sofia Global Settings</CardTitle></CardHeader><CardContent>
          <div className="hidden md:block"><Table><TableHeader><TableRow><TableHead>Name / ID</TableHead><TableHead>Details</TableHead></TableRow></TableHeader><TableBody>
                {data.map((d:any,i:number)=><TableRow key={i}><TableCell className="font-mono text-xs">{d["global_setting_name"] ?? d.name ?? JSON.stringify(d).slice(0,60)}</TableCell><TableCell className="text-xs text-muted-foreground">{JSON.stringify(d).slice(0,80)}</TableCell></TableRow>)}
                {data.length===0 && <TableRow><TableCell colSpan={2} className="text-center text-muted-foreground py-8">No sofia_global_settings — create via Add button POST /api/v1/sofia_global_settings</TableCell></TableRow>}
              </TableBody></Table></div>
          <div className="md:hidden grid gap-3">
            {data.map((d:any,i:number)=><Card key={i} className="p-3"><div className="font-mono text-xs break-all">{JSON.stringify(d).slice(0,120)}</div></Card>)}
            {data.length===0 && <div className="text-center text-muted-foreground py-4">No sofia_global_settings</div>}
          </div>
        </CardContent></Card>
      <Dialog open={open} onOpenChange={setOpen}><DialogHeader><DialogTitle>Add Sofia Global Settings</DialogTitle></DialogHeader>
        <div className="space-y-3"><div><Label>JSON payload *</Label><textarea value={json} onChange={(e)=>setJson(e.target.value)} rows={6} className="w-full rounded border p-2 font-mono text-xs" /></div>
          <div className="text-xs text-muted-foreground">Required field shown above — edit JSON and press Create.</div>
          <Button onClick={create} className="w-full">Create</Button></div></Dialog>
    </div>
  );
}
