import * as React from "react";
import { apiFetch } from "../lib/api";
import { Card, CardContent, CardHeader, CardTitle } from "../components/ui/card";
import { Button } from "../components/ui/button";
import { Input } from "../components/ui/input";
import { Label } from "../components/ui/label";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "../components/ui/table";
import { Dialog, DialogHeader, DialogTitle } from "../components/ui/dialog";
import { Badge } from "../components/ui/badge";

export function IvrMenus() {
  const [data, setData] = React.useState<any[]>([]);
  const [open, setOpen] = React.useState(false);
  const [editing, setEditing] = React.useState<any | null>(null);
  const [form, setForm] = React.useState({ name: "main-ivr", extension: "5000", options: "1:transfer:1000 XML $${domain},2:transfer:1001 XML $${domain}", enabled: true });

  const load = () => apiFetch("/ivr_menus").then((r) => setData(r.data || [])).catch(() => {});
  React.useEffect(() => { load(); }, []);

  const startCreate = () => { setEditing(null); setForm({ name: "main-ivr", extension: "5000", options: "1:transfer:1000 XML $${domain},2:transfer:1001 XML $${domain}", enabled: true }); setOpen(true); };
  const startEdit = (m: any) => {
    setEditing(m);
    const opts = (m.options || []).map((o: any) => `${o.digits}:${o.action}:${o.param}`).join(",");
    // fallback if options not in list data, keep placeholder
    setForm({ name: m.name || "", extension: m.extension || "", options: opts || "1:transfer:1000 XML $${domain}", enabled: !!m.enabled });
    setOpen(true);
  };
  const createOrUpdate = async () => {
    try {
      const opts = form.options.split(",").map((s) => { const [digits, action, param] = s.split(":"); return { digits, action, param }; });
      if (editing) {
        await apiFetch(`/ivr_menus/${editing.ivr_menu_uuid}`, { method: "PATCH", body: JSON.stringify({ name: form.name, extension: form.extension, enabled: form.enabled, options: opts }) });
      } else {
        await apiFetch("/ivr_menus", { method: "POST", body: JSON.stringify({ name: form.name, extension: form.extension, enabled: form.enabled, options: opts }) });
      }
      setOpen(false); setEditing(null); load();
    } catch (e: any) { alert(e.message); }
  };
  const toggleEnabled = async (m: any) => {
    try { await apiFetch(`/ivr_menus/${m.ivr_menu_uuid}`, { method: "PATCH", body: JSON.stringify({ enabled: !m.enabled }) }); load(); } catch (e: any) { alert(e.message); }
  };
  const deleteIvr = async (m: any) => {
    if (!confirm(`Delete IVR ${m.name}?`)) return;
    try { await apiFetch(`/ivr_menus/${m.ivr_menu_uuid}`, { method: "DELETE" }); load(); } catch (e: any) { alert(e.message); }
  };

  return (
    <div className="p-4 space-y-4">
      <div className="flex justify-between items-center"><h1 className="text-2xl font-bold">IVR Menus</h1><Button onClick={startCreate}>Add IVR</Button></div>
      <Card>
        <CardHeader><CardTitle>IVR Builder — XML via lxml</CardTitle></CardHeader>
        <CardContent>
          <div className="hidden md:block">
            <Table>
              <TableHeader><TableRow><TableHead>Name</TableHead><TableHead>Extension</TableHead><TableHead>Enabled</TableHead><TableHead>Actions</TableHead></TableRow></TableHeader>
              <TableBody>
                {data.map((m) => <TableRow key={m.ivr_menu_uuid}><TableCell>{m.name}</TableCell><TableCell className="font-mono">{m.extension || "-"}</TableCell><TableCell>{m.enabled ? <Badge>enabled</Badge> : <Badge className="bg-muted text-muted-foreground">disabled</Badge>}</TableCell><TableCell><div className="flex gap-1"><Button variant="outline" className="h-8" onClick={() => startEdit(m)}>Edit</Button><Button variant="outline" className="h-8" onClick={() => toggleEnabled(m)}>{m.enabled ? "Disable" : "Enable"}</Button><Button variant="outline" className="h-8 text-red-600 border-red-200" onClick={() => deleteIvr(m)}>Delete</Button></div></TableCell></TableRow>)}
                {data.length===0 && <TableRow><TableCell colSpan={4} className="text-center text-muted-foreground py-8">No IVR menus</TableCell></TableRow>}
              </TableBody>
            </Table>
          </div>
          <div className="md:hidden grid gap-3">
            {data.map((m) => <Card key={m.ivr_menu_uuid} className="p-3"><div className="flex justify-between"><div className="font-bold">{m.name}</div><Badge className={m.enabled ? "" : "bg-muted text-muted-foreground"}>{m.enabled ? "enabled" : "disabled"}</Badge></div><div className="text-xs font-mono">{m.extension}</div><div className="flex gap-2 mt-2"><Button variant="outline" className="flex-1 h-8" onClick={() => startEdit(m)}>Edit</Button><Button variant="outline" className="flex-1 h-8" onClick={() => toggleEnabled(m)}>{m.enabled ? "Disable" : "Enable"}</Button><Button variant="outline" className="flex-1 h-8 text-red-600" onClick={() => deleteIvr(m)}>Delete</Button></div></Card>)}
            {data.length===0 && <div className="text-center text-muted-foreground py-4">No IVR</div>}
          </div>
        </CardContent>
      </Card>
      <Dialog open={open} onOpenChange={(v) => { setOpen(v); if (!v) setEditing(null); }}>
        <DialogHeader><DialogTitle>{editing ? "Edit IVR" : "Add IVR"}</DialogTitle></DialogHeader>
        <div className="space-y-3">
          <div><Label>Name *</Label><Input value={form.name} onChange={(e) => setForm({ ...form, name: e.target.value })} /></div>
          <div><Label>Extension</Label><Input value={form.extension} onChange={(e) => setForm({ ...form, extension: e.target.value })} /></div>
          <div><Label>Options (digits:action:param comma separated)</Label><Input value={form.options} onChange={(e) => setForm({ ...form, options: e.target.value })} /></div>
          <div className="flex items-center gap-2"><input type="checkbox" id="ivr-enabled" checked={form.enabled} onChange={(e) => setForm({ ...form, enabled: e.target.checked })} className="h-4 w-4" /><Label htmlFor="ivr-enabled" className="cursor-pointer">{form.enabled ? "Enabled" : "Disabled"}</Label></div>
          <Button onClick={createOrUpdate} className="w-full">{editing ? "Save" : "Create"}</Button>
        </div>
      </Dialog>
    </div>
  );
}
