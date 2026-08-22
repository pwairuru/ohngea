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
  const [editing, setEditing] = React.useState<any | null>(null);
  const [form, setForm] = React.useState({ mac: "00:11:22:33:44:55", vendor: "poly", model: "VVX250", label: "", enabled: true });
  const [provision, setProvision] = React.useState<string>("");

  const load = () => apiFetch("/devices").then((r) => setData(r.data || [])).catch(() => {});
  React.useEffect(() => { load(); }, []);

  const startCreate = () => { setEditing(null); setForm({ mac: "00:11:22:33:44:55", vendor: "poly", model: "VVX250", label: "", enabled: true }); setOpen(true); };
  const startEdit = (d: any) => { setEditing(d); setForm({ mac: d.mac || "", vendor: d.vendor || "poly", model: d.model || "", label: d.label || "", enabled: !!d.enabled }); setOpen(true); };
  const createOrUpdate = async () => {
    try {
      if (editing) await apiFetch(`/devices/${editing.device_uuid}`, { method: "PATCH", body: JSON.stringify(form) });
      else await apiFetch("/devices", { method: "POST", body: JSON.stringify(form) });
      setOpen(false); setEditing(null); load();
    } catch (e: any) { alert(e.message); }
  };
  const toggleEnabled = async (d: any) => {
    try { await apiFetch(`/devices/${d.device_uuid}`, { method: "PATCH", body: JSON.stringify({ enabled: !d.enabled }) }); load(); } catch (e: any) { alert(e.message); }
  };
  const deleteDevice = async (d: any) => {
    if (!confirm(`Delete device ${d.mac}?`)) return;
    try { await apiFetch(`/devices/${d.device_uuid}`, { method: "DELETE" }); load(); } catch (e: any) { alert(e.message); }
  };
  const preview = async (mac: string, vendor: string) => {
    try { const txt = await fetch(`/provision/${mac}.cfg`).then((r) => r.text()); setProvision(txt); } catch { setProvision(`# Provision for ${vendor} mac ${mac}\n(preview fallback)`); }
  };

  return (
    <div className="p-4 space-y-4">
      <div className="flex justify-between items-center"><h1 className="text-2xl font-bold">Devices</h1><Button onClick={startCreate}>Add Device</Button></div>
      <Card>
        <CardHeader><CardTitle>Provisioned Devices</CardTitle></CardHeader>
        <CardContent>
          <div className="hidden md:block">
            <Table>
              <TableHeader><TableRow><TableHead>MAC</TableHead><TableHead>Vendor</TableHead><TableHead>Model</TableHead><TableHead>Enabled</TableHead><TableHead>Actions</TableHead></TableRow></TableHeader>
              <TableBody>
                {data.map((d) => <TableRow key={d.device_uuid}><TableCell className="font-mono">{d.mac}</TableCell><TableCell><Badge>{d.vendor}</Badge></TableCell><TableCell>{d.model || "-"}</TableCell><TableCell>{d.enabled ? <Badge>enabled</Badge> : <Badge className="bg-muted text-muted-foreground">disabled</Badge>}</TableCell><TableCell><div className="flex gap-1"><Button variant="outline" className="h-8" onClick={() => preview(d.mac, d.vendor)}>Preview</Button><Button variant="outline" className="h-8" onClick={() => startEdit(d)}>Edit</Button><Button variant="outline" className="h-8" onClick={() => toggleEnabled(d)}>{d.enabled ? "Disable" : "Enable"}</Button><Button variant="outline" className="h-8 text-red-600 border-red-200" onClick={() => deleteDevice(d)}>Delete</Button></div></TableCell></TableRow>)}
                {data.length===0 && <TableRow><TableCell colSpan={5} className="text-center text-muted-foreground py-8">No devices</TableCell></TableRow>}
              </TableBody>
            </Table>
          </div>
          <div className="md:hidden grid gap-3">
            {data.map((d) => <Card key={d.device_uuid} className="p-3"><div className="flex justify-between"><div className="font-mono font-bold">{d.mac}</div><Badge className={d.enabled ? "" : "bg-muted text-muted-foreground"}>{d.enabled ? "enabled" : "disabled"}</Badge></div><div className="text-xs">{d.vendor} {d.model}</div><div className="flex gap-2 mt-2"><Button variant="outline" className="flex-1 h-8" onClick={() => preview(d.mac, d.vendor)}>Preview</Button><Button variant="outline" className="flex-1 h-8" onClick={() => startEdit(d)}>Edit</Button></div><div className="flex gap-2 mt-2"><Button variant="outline" className="flex-1 h-8" onClick={() => toggleEnabled(d)}>{d.enabled ? "Disable" : "Enable"}</Button><Button variant="outline" className="flex-1 h-8 text-red-600" onClick={() => deleteDevice(d)}>Delete</Button></div></Card>)}
            {data.length===0 && <div className="text-center text-muted-foreground py-4">No devices</div>}
          </div>
          {provision && <pre className="mt-4 p-3 bg-muted rounded text-xs overflow-auto max-h-48">{provision}</pre>}
        </CardContent>
      </Card>
      <Dialog open={open} onOpenChange={(v) => { setOpen(v); if (!v) setEditing(null); }}>
        <DialogHeader><DialogTitle>{editing ? "Edit Device" : "Add Device"}</DialogTitle></DialogHeader>
        <div className="space-y-3">
          <div><Label>MAC *</Label><Input value={form.mac} onChange={(e) => setForm({ ...form, mac: e.target.value })} placeholder="00:11:22:33:44:55" /></div>
          <div className="grid grid-cols-2 gap-2"><div><Label>Vendor</Label><Input value={form.vendor} onChange={(e) => setForm({ ...form, vendor: e.target.value })} placeholder="poly/yealink" /></div><div><Label>Model</Label><Input value={form.model} onChange={(e) => setForm({ ...form, model: e.target.value })} /></div></div>
          <div><Label>Label</Label><Input value={form.label} onChange={(e) => setForm({ ...form, label: e.target.value })} placeholder="Desk phone" /></div>
          <div className="flex items-center gap-2"><input type="checkbox" id="dev-enabled" checked={form.enabled} onChange={(e) => setForm({ ...form, enabled: e.target.checked })} className="h-4 w-4" /><Label htmlFor="dev-enabled" className="cursor-pointer">{form.enabled ? "Enabled" : "Disabled"}</Label></div>
          <Button onClick={createOrUpdate} className="w-full">{editing ? "Save" : "Create"}</Button>
        </div>
      </Dialog>
    </div>
  );
}
