import * as React from "react";
import { apiFetch } from "../lib/api";
import { Card, CardContent, CardHeader, CardTitle } from "../components/ui/card";
import { Button } from "../components/ui/button";
import { Input } from "../components/ui/input";
import { Label } from "../components/ui/label";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "../components/ui/table";
import { Dialog, DialogHeader, DialogTitle } from "../components/ui/dialog";
import { Badge } from "../components/ui/badge";

export function Devices() {
  const [data, setData] = React.useState<any[]>([]);
  const [open, setOpen] = React.useState(false);
  const [form, setForm] = React.useState({ mac: "00:11:22:33:44:55", vendor: "poly", model: "VVX250" });
  const [provision, setProvision] = React.useState<string>("");

  const load = () => apiFetch("/devices").then((r) => setData(r.data || [])).catch(() => {});
  React.useEffect(() => { load(); }, []);

  const create = async () => {
    try { await apiFetch("/devices", { method: "POST", body: JSON.stringify(form) }); setOpen(false); load(); } catch (e: any) { alert(e.message); }
  };
  const preview = async (mac: string, vendor: string) => {
    try { const txt = await fetch(`/provision/${mac}.cfg`).then((r) => r.text()); setProvision(txt); } catch { setProvision(`# Provision for ${vendor} mac ${mac}\n(preview fallback)`); }
  };

  return (
    <div className="p-4 space-y-4">
      <div className="flex justify-between items-center"><h1 className="text-2xl font-bold">Devices</h1><Button onClick={() => setOpen(true)}>Add Device</Button></div>
      <Card>
        <CardHeader><CardTitle>Provisioned Devices</CardTitle></CardHeader>
        <CardContent>
          <div className="hidden md:block">
            <Table>
              <TableHeader><TableRow><TableHead>MAC</TableHead><TableHead>Vendor</TableHead><TableHead>Model</TableHead><TableHead>Provision</TableHead></TableRow></TableHeader>
              <TableBody>
                {data.map((d) => <TableRow key={d.device_uuid}><TableCell className="font-mono">{d.mac}</TableCell><TableCell><Badge>{d.vendor}</Badge></TableCell><TableCell>{d.model || "-"}</TableCell><TableCell><Button variant="outline" className="h-8" onClick={() => preview(d.mac, d.vendor)}>Preview</Button></TableCell></TableRow>)}
                {data.length===0 && <TableRow><TableCell colSpan={4} className="text-center text-muted-foreground py-8">No devices</TableCell></TableRow>}
              </TableBody>
            </Table>
          </div>
          <div className="md:hidden grid gap-3">
            {data.map((d) => <Card key={d.device_uuid} className="p-3"><div className="font-mono font-bold">{d.mac}</div><div className="text-xs">{d.vendor} {d.model}</div><Button variant="outline" className="mt-2 w-full" onClick={() => preview(d.mac, d.vendor)}>Provision Preview</Button></Card>)}
            {data.length===0 && <div className="text-center text-muted-foreground py-4">No devices</div>}
          </div>
          {provision && <pre className="mt-4 p-3 bg-muted rounded text-xs overflow-auto max-h-48">{provision}</pre>}
        </CardContent>
      </Card>
      <Dialog open={open} onOpenChange={setOpen}>
        <DialogHeader><DialogTitle>Add Device</DialogTitle></DialogHeader>
        <div className="space-y-3">
          <div><Label>MAC *</Label><Input value={form.mac} onChange={(e) => setForm({ ...form, mac: e.target.value })} placeholder="00:11:22:33:44:55" /></div>
          <div className="grid grid-cols-2 gap-2"><div><Label>Vendor</Label><Input value={form.vendor} onChange={(e) => setForm({ ...form, vendor: e.target.value })} placeholder="poly/yealink" /></div><div><Label>Model</Label><Input value={form.model} onChange={(e) => setForm({ ...form, model: e.target.value })} /></div></div>
          <Button onClick={create} className="w-full">Create</Button>
        </div>
      </Dialog>
    </div>
  );
}
