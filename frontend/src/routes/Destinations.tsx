import * as React from "react";
import { apiFetch } from "../lib/api";
import { Card, CardContent, CardHeader, CardTitle } from "../components/ui/card";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "../components/ui/table";
import { Badge } from "../components/ui/badge";
import { Button } from "../components/ui/button";
import { Input } from "../components/ui/input";
import { Label } from "../components/ui/label";
import { Dialog, DialogHeader, DialogTitle } from "../components/ui/dialog";
import {Pencil, Power, PowerOff, Trash2, ChevronsLeft, ChevronLeft, ChevronRight, ChevronsRight} from "lucide-react";

export function Destinations() {
  const [data, setData] = React.useState<any[]>([]);
  const [total, setTotal] = React.useState(0);
  const [q, setQ] = React.useState("");
  const [page, setPage] = React.useState(1);
  const [open, setOpen] = React.useState(false);
  const [editing, setEditing] = React.useState<any | null>(null);
  const [form, setForm] = React.useState({
    destination_number: "1001",
    destination_type: "inbound",
    destination_description: "",
    destination_context: "public",
    destination_app: "",
    destination_data: "",
    destination_enabled: true,
  });

  const load = React.useCallback(() => {
    const params = new URLSearchParams({ page: String(page), per_page: "20" });
    if (q) params.set("q", q);
    apiFetch(`/destinations?${params.toString()}`)
      .then((r: any) => {
        setData(r.data || []);
        setTotal(r.total ?? r.data?.length ?? 0);
      })
      .catch(() => {});
  }, [page, q]);

  React.useEffect(() => { load(); }, [load]);

  const startCreate = () => {
    setEditing(null);
    setForm({ destination_number: "1001", destination_type: "inbound", destination_description: "", destination_context: "public", destination_app: "", destination_data: "", destination_enabled: true });
    setOpen(true);
  };

  const startEdit = (row: any) => {
    setEditing(row);
    setForm({
      destination_number: row.destination_number || row.number || "",
      destination_type: row.destination_type || "inbound",
      destination_description: row.destination_description || row.description || "",
      destination_context: row.destination_context || "public",
      destination_app: row.destination_app || "",
      destination_data: row.destination_data || "",
      destination_enabled: row.destination_enabled ?? row.enabled ?? true,
    });
    setOpen(true);
  };

  const createOrUpdate = async () => {
    try {
      const payload: any = {
        destination_number: form.destination_number,
        destination_type: form.destination_type,
        destination_enabled: form.destination_enabled,
        destination_description: form.destination_description || null,
        destination_context: form.destination_context || null,
        destination_app: form.destination_app || null,
        destination_data: form.destination_data || null,
        // aliases
        number: form.destination_number,
        enabled: form.destination_enabled,
        description: form.destination_description || null,
      };
      if (editing) {
        const uid = editing.destination_uuid || editing.id;
        await apiFetch(`/destinations/${uid}`, { method: "PATCH", body: JSON.stringify(payload) });
      } else {
        await apiFetch("/destinations", { method: "POST", body: JSON.stringify(payload) });
      }
      setOpen(false); setEditing(null); load();
    } catch (e: any) { alert(e.message); }
  };

  const del = async (row: any) => {
    const uid = row.destination_uuid || row.id;
    if (!confirm(`Delete destination ${row.destination_number || uid}?`)) return;
    try { await apiFetch(`/destinations/${uid}`, { method: "DELETE" }); load(); } catch (e: any) { alert(e.message); }
  };

  const toggleEnabled = async (row: any) => {
    const uid = row.destination_uuid || row.id;
    const enabled = row.destination_enabled ?? row.enabled;
    try { await apiFetch(`/destinations/${uid}`, { method: "PATCH", body: JSON.stringify({ destination_enabled: !enabled, enabled: !enabled }) }); load(); } catch (e: any) { alert(e.message); }
  };

  return (
    <div className="p-4 space-y-4">
      <div className="flex justify-between items-center"><h1 className="text-2xl font-bold">Destinations</h1><div className="flex gap-2"><Badge>{total} total</Badge><Button onClick={startCreate}>Add Destination</Button></div></div>

      <div className="flex gap-2">
        <Input value={q} onChange={(e) => setQ(e.target.value)} placeholder="Search number, description..." className="flex-1" />
        <Button variant="outline" onClick={() => { setPage(1); load(); }}>Search</Button>
      </div>

      <Card><CardHeader><CardTitle>Destinations — inbound routing</CardTitle></CardHeader><CardContent>
          <div className="hidden md:block"><Table><TableHeader><TableRow><TableHead>Number</TableHead><TableHead>Type</TableHead><TableHead>Context</TableHead><TableHead>Description</TableHead><TableHead>Enabled</TableHead><TableHead>Actions</TableHead></TableRow></TableHeader><TableBody>
                {data.map((d: any) => {
                  const uid = d.destination_uuid || d.id;
                  return (
                    <TableRow key={String(uid)}>
                      <TableCell className="font-mono font-bold">{d.destination_number ?? d.number}</TableCell>
                      <TableCell className="text-xs">{d.destination_type}</TableCell>
                      <TableCell className="text-xs">{d.destination_context || "-"}</TableCell>
                      <TableCell className="text-xs text-muted-foreground max-w-[200px] truncate" title={d.destination_description || d.description}>{d.destination_description || d.description || "-"}</TableCell>
                      <TableCell>{(d.destination_enabled ?? d.enabled) ? <Badge>enabled</Badge> : <Badge className="bg-muted text-muted-foreground">disabled</Badge>}</TableCell>
                      <TableCell><div className="flex gap-1"><Button variant="ghost" aria-label={`Edit ${d.destination_number ?? d.number}`} title="Edit" className="h-9 w-9 p-0" onClick={() => startEdit(d)}><Pencil className="h-4 w-4" /></Button><Button variant="ghost" aria-label={(d.destination_enabled ?? d.enabled) ? `Disable ${d.destination_number}` : `Enable ${d.destination_number}`} title={(d.destination_enabled ?? d.enabled) ? "Disable" : "Enable"} className="h-9 w-9 p-0" onClick={() => toggleEnabled(d)}>{(d.destination_enabled ?? d.enabled) ? <PowerOff className="h-4 w-4" /> : <Power className="h-4 w-4" />}</Button><Button variant="ghost" aria-label={`Delete ${d.destination_number}`} title="Delete" className="h-9 w-9 p-0 text-destructive hover:text-destructive hover:bg-destructive/10" onClick={() => del(d)}><Trash2 className="h-4 w-4" /></Button></div></TableCell>
                    </TableRow>
                  );
                })}
                {data.length===0 && <TableRow><TableCell colSpan={6} className="text-center text-muted-foreground py-8">No destinations — create via Add button</TableCell></TableRow>}
              </TableBody></Table></div>
          <div className="md:hidden grid gap-3">
            {data.map((d: any) => {
              const uid = d.destination_uuid || d.id;
              return (
                <Card key={String(uid)} className="p-3 space-y-1">
                  <div className="flex justify-between"><div className="font-mono font-bold">{d.destination_number}</div><Badge className={(d.destination_enabled ?? d.enabled) ? "" : "bg-muted text-muted-foreground"}>{(d.destination_enabled ?? d.enabled) ? "enabled" : "disabled"}</Badge></div>
                  <div className="text-xs">{d.destination_type} • {d.destination_context}</div>
                  <div className="text-xs text-muted-foreground">{d.destination_description || "no description"}</div>
                  <div className="flex gap-2 mt-2"><Button variant="ghost" aria-label={`Edit ${d.destination_number}`} title="Edit" className="flex-1 h-9" onClick={() => startEdit(d)}><Pencil className="h-4 w-4" /></Button><Button variant="ghost" aria-label={(d.destination_enabled ?? d.enabled) ? `Disable ${d.destination_number}` : `Enable ${d.destination_number}`} title={(d.destination_enabled ?? d.enabled) ? "Disable" : "Enable"} className="flex-1 h-9" onClick={() => toggleEnabled(d)}>{(d.destination_enabled ?? d.enabled) ? <PowerOff className="h-4 w-4" /> : <Power className="h-4 w-4" />}</Button><Button variant="ghost" aria-label={`Delete ${d.destination_number}`} title="Delete" className="flex-1 h-9 text-destructive hover:text-destructive hover:bg-destructive/10" onClick={() => del(d)}><Trash2 className="h-4 w-4" /></Button></div>
                </Card>
              );
            })}
            {data.length===0 && <div className="text-center text-muted-foreground py-4">No destinations</div>}
          </div>
          <div className="flex items-center justify-between mt-4 text-sm">
            <span>Total: {total}</span>
            <div className="flex items-center gap-1">
              <Button variant="ghost" size="icon" aria-label="First page" title="First page" className="h-9 w-9 p-0" disabled={page <= 1} onClick={() => setPage(1)}><ChevronsLeft className="h-4 w-4" /></Button>
              <Button variant="ghost" size="icon" aria-label="Previous page" title="Previous page" className="h-9 w-9 p-0" disabled={page <= 1} onClick={() => setPage((p) => Math.max(1, p - 1))}><ChevronLeft className="h-4 w-4" /></Button>
              <span className="px-2 text-sm">Page {page} of {Math.max(1, Math.ceil(total / 20))}</span>
              <Button variant="ghost" size="icon" aria-label="Next page" title="Next page" className="h-9 w-9 p-0" disabled={page * 20 >= total} onClick={() => setPage((p) => p + 1)}><ChevronRight className="h-4 w-4" /></Button>
              <Button variant="ghost" size="icon" aria-label="Last page" title="Last page" className="h-9 w-9 p-0" disabled={page * 20 >= total} onClick={() => setPage(Math.max(1, Math.ceil(total / 20)))}><ChevronsRight className="h-4 w-4" /></Button>
            </div>
          </div>
        </CardContent></Card>
      <Dialog open={open} onOpenChange={(v) => { setOpen(v); if (!v) setEditing(null); }}><DialogHeader><DialogTitle>{editing ? "Edit Destination" : "Add Destination"}</DialogTitle></DialogHeader>
        <div className="space-y-3">
          <div className="grid grid-cols-2 gap-2">
            <div><Label>Number *</Label><Input value={form.destination_number} onChange={(e) => setForm({ ...form, destination_number: e.target.value })} placeholder="1001" /></div>
            <div><Label>Type</Label><select value={form.destination_type} onChange={(e) => setForm({ ...form, destination_type: e.target.value })} className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm min-h-[44px]"><option value="inbound">inbound</option><option value="outbound">outbound</option></select></div>
          </div>
          <div className="grid grid-cols-2 gap-2">
            <div><Label>Context</Label><Input value={form.destination_context} onChange={(e) => setForm({ ...form, destination_context: e.target.value })} placeholder="public" /></div>
            <div className="flex items-center gap-2 pt-6"><input type="checkbox" id="dest-enabled" checked={form.destination_enabled} onChange={(e) => setForm({ ...form, destination_enabled: e.target.checked })} className="h-4 w-4" /><Label htmlFor="dest-enabled" className="cursor-pointer">Enabled</Label></div>
          </div>
          <div><Label>Description</Label><Input value={form.destination_description} onChange={(e) => setForm({ ...form, destination_description: e.target.value })} placeholder="Main DID" /></div>
          <div className="grid grid-cols-2 gap-2"><div><Label>App</Label><Input value={form.destination_app} onChange={(e) => setForm({ ...form, destination_app: e.target.value })} placeholder="transfer" /></div><div><Label>Data</Label><Input value={form.destination_data} onChange={(e) => setForm({ ...form, destination_data: e.target.value })} placeholder="1001 XML default" /></div></div>
          <Button onClick={createOrUpdate} className="w-full">{editing ? "Save" : "Create"}</Button>
        </div></Dialog>
    </div>
  );
}
