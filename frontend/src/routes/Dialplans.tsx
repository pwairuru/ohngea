import * as React from "react";
import { apiFetch } from "../lib/api";
import { Card, CardContent, CardHeader, CardTitle } from "../components/ui/card";
import { Button } from "../components/ui/button";
import { Input } from "../components/ui/input";
import { Label } from "../components/ui/label";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "../components/ui/table";
import { Dialog, DialogHeader, DialogTitle } from "../components/ui/dialog";
import { Badge } from "../components/ui/badge";

export function Dialplans() {
  const [data, setData] = React.useState<any[]>([]);
  const [open, setOpen] = React.useState(false);
  const [form, setForm] = React.useState({ name: "", context: "", destination: "^1000$", action: "transfer", data: "1000 XML $${domain}" });

  const load = () => apiFetch("/dialplans").then((r) => setData(r.data || [])).catch(() => {});
  React.useEffect(() => { load(); }, []);

  const create = async () => {
    try {
      const domains = await apiFetch("/domains");
      const domain = domains[0]?.domain_name || domains.data?.[0]?.domain_name || form.context;
      await apiFetch("/dialplans", { method: "POST", body: JSON.stringify({ name: form.name, context: domain, enabled: true, details: [{ tag: "condition", type: "destination_number", data: form.destination }, { tag: "action", type: form.action, data: form.data }] }) });
      setOpen(false); load();
    } catch (e: any) { alert(e.message); }
  };

  return (
    <div className="p-4 space-y-4">
      <div className="flex justify-between items-center"><h1 className="text-2xl font-bold">Call Routing</h1><Button onClick={() => setOpen(true)}>Add Dialplan</Button></div>
      <Card>
        <CardHeader><CardTitle>Dialplans (visual flow)</CardTitle></CardHeader>
        <CardContent>
          <div className="hidden md:block">
            <Table>
              <TableHeader><TableRow><TableHead>Name</TableHead><TableHead>Context</TableHead><TableHead>Enabled</TableHead></TableRow></TableHeader>
              <TableBody>
                {data.map((d) => <TableRow key={d.dialplan_uuid}><TableCell className="font-mono">{d.name}</TableCell><TableCell>{d.context}</TableCell><TableCell>{d.enabled ? <Badge>yes</Badge> : "no"}</TableCell></TableRow>)}
                {data.length===0 && <TableRow><TableCell colSpan={3} className="text-center text-muted-foreground py-8">No dialplans — create 3-step wizard flow</TableCell></TableRow>}
              </TableBody>
            </Table>
          </div>
          <div className="md:hidden grid gap-3">
            {data.map((d) => <Card key={d.dialplan_uuid} className="p-3"><div className="font-mono font-bold">{d.name}</div><div className="text-xs text-muted-foreground">{d.context}</div></Card>)}
            {data.length===0 && <div className="text-center text-muted-foreground py-4">No dialplans</div>}
          </div>
        </CardContent>
      </Card>
      <Card>
        <CardHeader><CardTitle>Wizard: Extensions → Trunks → Routes</CardTitle></CardHeader>
        <CardContent className="text-sm text-muted-foreground">
          Cudatel/Barracuda inspired 3-step: 1) Pick extension 1000, 2) Pick gateway, 3) Test dial — validates transfer.
        </CardContent>
      </Card>

      <Dialog open={open} onOpenChange={setOpen}>
        <DialogHeader><DialogTitle>Add Dialplan</DialogTitle></DialogHeader>
        <div className="space-y-3">
          <div><Label>Name *</Label><Input value={form.name} onChange={(e) => setForm({ ...form, name: e.target.value })} placeholder="inbound-1000" /></div>
          <div><Label>Context</Label><Input value={form.context} onChange={(e) => setForm({ ...form, context: e.target.value })} placeholder="tenant.example.com" /></div>
          <div><Label>Destination regex</Label><Input value={form.destination} onChange={(e) => setForm({ ...form, destination: e.target.value })} /></div>
          <div className="grid grid-cols-2 gap-2"><div><Label>Action</Label><Input value={form.action} onChange={(e) => setForm({ ...form, action: e.target.value })} /></div><div><Label>Data</Label><Input value={form.data} onChange={(e) => setForm({ ...form, data: e.target.value })} /></div></div>
          <Button onClick={create} className="w-full">Create</Button>
        </div>
      </Dialog>
    </div>
  );
}
