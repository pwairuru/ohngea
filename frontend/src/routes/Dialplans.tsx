import * as React from "react";
import { apiFetch } from "../lib/api";
import { Card, CardContent, CardHeader, CardTitle } from "../components/ui/card";
import { Button } from "../components/ui/button";
import { Input } from "../components/ui/input";
import { Label } from "../components/ui/label";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "../components/ui/table";
import { Dialog, DialogHeader, DialogTitle } from "../components/ui/dialog";
import { Badge } from "../components/ui/badge";
import { Pencil, Power, PowerOff, Trash2 } from "lucide-react";

export function Dialplans() {
  const [data, setData] = React.useState<any[]>([]);
  const [open, setOpen] = React.useState(false);
  const [editing, setEditing] = React.useState<any | null>(null);
  const [form, setForm] = React.useState({ name: "", context: "", destination: "^1000$", action: "transfer", data: "1000 XML $${domain}", enabled: true });

  const load = () => apiFetch("/dialplans").then((r) => setData(r.data || [])).catch(() => {});
  React.useEffect(() => { load(); }, []);

  const startCreate = () => { setEditing(null); setForm({ name: "", context: "", destination: "^1000$", action: "transfer", data: "1000 XML $${domain}", enabled: true }); setOpen(true); };
  const startEdit = (d: any) => {
    setEditing(d);
    // try to extract destination/action from details if present
    const details = d.details || [];
    const cond = details.find((x: any) => x.tag === "condition");
    const act = details.find((x: any) => x.tag === "action");
    setForm({
      name: d.name || "",
      context: d.context || "",
      destination: cond?.data || "^1000$",
      action: act?.type || "transfer",
      data: act?.data || "1000 XML $${domain}",
      enabled: !!d.enabled,
    });
    setOpen(true);
  };
  const createOrUpdate = async () => {
    try {
      if (editing) {
        await apiFetch(`/dialplans/${editing.dialplan_uuid}`, { method: "PATCH", body: JSON.stringify({ name: form.name, context: form.context, enabled: form.enabled }) });
      } else {
        let domain = form.context;
        if (!domain) {
          const me: any = await apiFetch("/auth/me");
          if (me.domain_id) {
            const d: any = await apiFetch(`/domains/${me.domain_id}`);
            domain = d.domain_name;
          }
        }
        await apiFetch("/dialplans", { method: "POST", body: JSON.stringify({ name: form.name, context: domain, enabled: form.enabled, details: [{ tag: "condition", type: "destination_number", data: form.destination }, { tag: "action", type: form.action, data: form.data }] }) });
      }
      setOpen(false); setEditing(null); load();
    } catch (e: any) { alert(e.message); }
  };
  const toggleEnabled = async (d: any) => {
    try { await apiFetch(`/dialplans/${d.dialplan_uuid}`, { method: "PATCH", body: JSON.stringify({ enabled: !d.enabled }) }); load(); } catch (e: any) { alert(e.message); }
  };
  const deleteDialplan = async (d: any) => {
    if (!confirm(`Delete dialplan ${d.name}?`)) return;
    try { await apiFetch(`/dialplans/${d.dialplan_uuid}`, { method: "DELETE" }); load(); } catch (e: any) { alert(e.message); }
  };

  return (
    <div className="p-4 space-y-4">
      <div className="flex justify-between items-center"><h1 className="text-2xl font-bold">Call Routing</h1><Button onClick={startCreate}>Add Dialplan</Button></div>
      <Card>
        <CardHeader><CardTitle>Dialplans (visual flow)</CardTitle></CardHeader>
        <CardContent>
          <div className="hidden md:block">
            <Table>
              <TableHeader><TableRow><TableHead>Name</TableHead><TableHead>Context</TableHead><TableHead>Enabled</TableHead><TableHead>Actions</TableHead></TableRow></TableHeader>
              <TableBody>
                {data.map((d) => <TableRow key={d.dialplan_uuid}><TableCell className="font-mono">{d.name}</TableCell><TableCell>{d.context}</TableCell><TableCell>{d.enabled ? <Badge>enabled</Badge> : <Badge className="bg-muted text-muted-foreground">disabled</Badge>}</TableCell><TableCell><div className="flex gap-1"><Button variant="ghost" aria-label={`Edit ${d.name}`} title="Edit" className="h-9 w-9 p-0" onClick={() => startEdit(d)}><Pencil className="h-4 w-4" /></Button><Button variant="ghost" aria-label={d.enabled ? `Disable ${d.name}` : `Enable ${d.name}`} title={d.enabled ? "Disable" : "Enable"} className="h-9 w-9 p-0" onClick={() => toggleEnabled(d)}>{d.enabled ? <PowerOff className="h-4 w-4" /> : <Power className="h-4 w-4" />}</Button><Button variant="ghost" aria-label={`Delete ${d.name}`} title="Delete" className="h-9 w-9 p-0 text-destructive hover:text-destructive hover:bg-destructive/10" onClick={() => deleteDialplan(d)}><Trash2 className="h-4 w-4" /></Button></div></TableCell></TableRow>)}
                {data.length===0 && <TableRow><TableCell colSpan={4} className="text-center text-muted-foreground py-8">No dialplans — create 3-step wizard flow</TableCell></TableRow>}
              </TableBody>
            </Table>
          </div>
          <div className="md:hidden grid gap-3">
            {data.map((d) => <Card key={d.dialplan_uuid} className="p-3"><div className="flex justify-between"><div className="font-mono font-bold">{d.name}</div><Badge className={d.enabled ? "" : "bg-muted text-muted-foreground"}>{d.enabled ? "enabled" : "disabled"}</Badge></div><div className="text-xs text-muted-foreground">{d.context}</div><div className="flex gap-2 mt-2"><Button variant="ghost" aria-label={`Edit ${d.name}`} title="Edit" className="flex-1 h-9" onClick={() => startEdit(d)}><Pencil className="h-4 w-4" /></Button><Button variant="ghost" aria-label={d.enabled ? `Disable ${d.name}` : `Enable ${d.name}`} title={d.enabled ? "Disable" : "Enable"} className="flex-1 h-9" onClick={() => toggleEnabled(d)}>{d.enabled ? <PowerOff className="h-4 w-4" /> : <Power className="h-4 w-4" />}</Button><Button variant="ghost" aria-label={`Delete ${d.name}`} title="Delete" className="flex-1 h-9 text-destructive hover:text-destructive hover:bg-destructive/10" onClick={() => deleteDialplan(d)}><Trash2 className="h-4 w-4" /></Button></div></Card>)}
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

      <Dialog open={open} onOpenChange={(v) => { setOpen(v); if (!v) setEditing(null); }}>
        <DialogHeader><DialogTitle>{editing ? "Edit Dialplan" : "Add Dialplan"}</DialogTitle></DialogHeader>
        <div className="space-y-3">
          <div><Label>Name *</Label><Input value={form.name} onChange={(e) => setForm({ ...form, name: e.target.value })} placeholder="inbound-1000" /></div>
          <div><Label>Context</Label><Input value={form.context} onChange={(e) => setForm({ ...form, context: e.target.value })} placeholder="tenant.example.com" /></div>
          {!editing && <>
            <div><Label>Destination regex</Label><Input value={form.destination} onChange={(e) => setForm({ ...form, destination: e.target.value })} /></div>
            <div className="grid grid-cols-2 gap-2"><div><Label>Action</Label><Input value={form.action} onChange={(e) => setForm({ ...form, action: e.target.value })} /></div><div><Label>Data</Label><Input value={form.data} onChange={(e) => setForm({ ...form, data: e.target.value })} /></div></div>
          </>}
          <div className="flex items-center gap-2"><input type="checkbox" id="dp-enabled" checked={form.enabled} onChange={(e) => setForm({ ...form, enabled: e.target.checked })} className="h-4 w-4" /><Label htmlFor="dp-enabled" className="cursor-pointer">{form.enabled ? "Enabled" : "Disabled"}</Label></div>
          <Button onClick={createOrUpdate} className="w-full">{editing ? "Save" : "Create"}</Button>
        </div>
      </Dialog>
    </div>
  );
}
