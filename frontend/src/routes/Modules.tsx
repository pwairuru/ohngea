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

export function Modules() {
  const [data, setData] = React.useState<any[]>([]);
  const [total, setTotal] = React.useState(0);
  const [q, setQ] = React.useState("");
  const [page, setPage] = React.useState(1);
  const [open, setOpen] = React.useState(false);
  const [editing, setEditing] = React.useState<any | null>(null);
  const [form, setForm] = React.useState({ module_label: "Test", module_name: "test_mod", module_category: "", module_description: "", module_enabled: true });

  const load = React.useCallback(() => {
    const params = new URLSearchParams({ page: String(page), per_page: "20" });
    if (q) params.set("q", q);
    apiFetch(`/modules?${params.toString()}`)
      .then((r: any) => { setData(r.data || []); setTotal(r.total ?? r.data?.length ?? 0); })
      .catch(() => {});
  }, [page, q]);

  React.useEffect(() => { load(); }, [load]);

  const startCreate = () => {
    setEditing(null);
    setForm({ module_label: "Test", module_name: "test_mod", module_category: "", module_description: "", module_enabled: true });
    setOpen(true);
  };

  const startEdit = (row: any) => {
    setEditing(row);
    setForm({
      module_label: row.module_label || row.label || "",
      module_name: row.module_name || row.name || "",
      module_category: row.module_category || "",
      module_description: row.module_description || row.description || "",
      module_enabled: row.module_enabled ?? row.enabled ?? true,
    });
    setOpen(true);
  };

  const createOrUpdate = async () => {
    try {
      const payload: any = {
        module_label: form.module_label,
        module_name: form.module_name,
        module_category: form.module_category || null,
        module_enabled: form.module_enabled,
        module_description: form.module_description || null,
        // aliases
        label: form.module_label,
        name: form.module_name,
        enabled: form.module_enabled,
        description: form.module_description || null,
      };
      if (editing) {
        const uid = editing.module_uuid || editing.id;
        await apiFetch(`/modules/${uid}`, { method: "PATCH", body: JSON.stringify(payload) });
      } else {
        await apiFetch("/modules", { method: "POST", body: JSON.stringify(payload) });
      }
      setOpen(false); setEditing(null); load();
    } catch (e: any) { alert(e.message); }
  };

  const del = async (row: any) => {
    const uid = row.module_uuid || row.id;
    if (!confirm(`Delete module ${row.module_name || uid}?`)) return;
    try { await apiFetch(`/modules/${uid}`, { method: "DELETE" }); load(); } catch (e: any) { alert(e.message); }
  };

  const toggleEnabled = async (row: any) => {
    const uid = row.module_uuid || row.id;
    const enabled = row.module_enabled ?? row.enabled;
    try { await apiFetch(`/modules/${uid}`, { method: "PATCH", body: JSON.stringify({ module_enabled: !enabled, enabled: !enabled }) }); load(); } catch (e: any) { alert(e.message); }
  };

  return (
    <div className="p-4 space-y-4">
      <div className="flex justify-between items-center"><h1 className="text-2xl font-bold">Modules</h1><div className="flex gap-2"><Badge>{total} total</Badge><Button onClick={startCreate}>Add Module</Button></div></div>

      <div className="flex gap-2">
        <Input value={q} onChange={(e) => setQ(e.target.value)} placeholder="Search label, name..." className="flex-1" />
        <Button variant="outline" onClick={() => { setPage(1); load(); }}>Search</Button>
      </div>

      <Card><CardHeader><CardTitle>Modules</CardTitle></CardHeader><CardContent>
          <div className="hidden md:block"><Table><TableHeader><TableRow><TableHead>Label</TableHead><TableHead>Name</TableHead><TableHead>Category</TableHead><TableHead>Description</TableHead><TableHead>Enabled</TableHead><TableHead>Actions</TableHead></TableRow></TableHeader><TableBody>
                {data.map((d: any) => {
                  const uid = d.module_uuid || d.id;
                  return (
                    <TableRow key={String(uid)}>
                      <TableCell className="font-mono font-bold">{d.module_label ?? d.label}</TableCell>
                      <TableCell className="font-mono text-xs">{d.module_name ?? d.name}</TableCell>
                      <TableCell className="text-xs">{d.module_category || "-"}</TableCell>
                      <TableCell className="text-xs text-muted-foreground max-w-[200px] truncate" title={d.module_description || d.description}>{d.module_description || d.description || "-"}</TableCell>
                      <TableCell>{(d.module_enabled ?? d.enabled) ? <Badge>enabled</Badge> : <Badge className="bg-muted text-muted-foreground">disabled</Badge>}</TableCell>
                      <TableCell><div className="flex gap-1"><Button variant="ghost" aria-label={`Edit ${d.module_label ?? d.label}`} title="Edit" className="h-9 w-9 p-0" onClick={() => startEdit(d)}><Pencil className="h-4 w-4" /></Button><Button variant="ghost" aria-label={(d.module_enabled ?? d.enabled) ? `Disable ${d.module_label}` : `Enable ${d.module_label}`} title={(d.module_enabled ?? d.enabled) ? "Disable" : "Enable"} className="h-9 w-9 p-0" onClick={() => toggleEnabled(d)}>{(d.module_enabled ?? d.enabled) ? <PowerOff className="h-4 w-4" /> : <Power className="h-4 w-4" />}</Button><Button variant="ghost" aria-label={`Delete ${d.module_label}`} title="Delete" className="h-9 w-9 p-0 text-destructive hover:text-destructive hover:bg-destructive/10" onClick={() => del(d)}><Trash2 className="h-4 w-4" /></Button></div></TableCell>
                    </TableRow>
                  );
                })}
                {data.length===0 && <TableRow><TableCell colSpan={6} className="text-center text-muted-foreground py-8">No modules — create via Add button</TableCell></TableRow>}
              </TableBody></Table></div>
          <div className="md:hidden grid gap-3">
            {data.map((d: any) => {
              const uid = d.module_uuid || d.id;
              return (
                <Card key={String(uid)} className="p-3 space-y-1">
                  <div className="flex justify-between"><div className="font-mono font-bold">{d.module_label}</div><Badge className={(d.module_enabled ?? d.enabled) ? "" : "bg-muted text-muted-foreground"}>{(d.module_enabled ?? d.enabled) ? "enabled" : "disabled"}</Badge></div>
                  <div className="text-xs font-mono">{d.module_name} • {d.module_category || "no category"}</div>
                  <div className="text-xs text-muted-foreground">{d.module_description || "no description"}</div>
                  <div className="flex gap-2 mt-2"><Button variant="ghost" aria-label={`Edit ${d.module_label}`} title="Edit" className="flex-1 h-9" onClick={() => startEdit(d)}><Pencil className="h-4 w-4" /></Button><Button variant="ghost" aria-label={(d.module_enabled ?? d.enabled) ? `Disable ${d.module_label}` : `Enable ${d.module_label}`} title={(d.module_enabled ?? d.enabled) ? "Disable" : "Enable"} className="flex-1 h-9" onClick={() => toggleEnabled(d)}>{(d.module_enabled ?? d.enabled) ? <PowerOff className="h-4 w-4" /> : <Power className="h-4 w-4" />}</Button><Button variant="ghost" aria-label={`Delete ${d.module_label}`} title="Delete" className="flex-1 h-9 text-destructive hover:text-destructive hover:bg-destructive/10" onClick={() => del(d)}><Trash2 className="h-4 w-4" /></Button></div>
                </Card>
              );
            })}
            {data.length===0 && <div className="text-center text-muted-foreground py-4">No modules</div>}
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
      <Dialog open={open} onOpenChange={(v) => { setOpen(v); if (!v) setEditing(null); }}><DialogHeader><DialogTitle>{editing ? "Edit Module" : "Add Module"}</DialogTitle></DialogHeader>
        <div className="space-y-3">
          <div className="grid grid-cols-2 gap-2">
            <div><Label>Label *</Label><Input value={form.module_label} onChange={(e) => setForm({ ...form, module_label: e.target.value })} placeholder="Test" /></div>
            <div><Label>Name *</Label><Input value={form.module_name} onChange={(e) => setForm({ ...form, module_name: e.target.value })} placeholder="test_mod" /></div>
          </div>
          <div><Label>Category</Label><Input value={form.module_category} onChange={(e) => setForm({ ...form, module_category: e.target.value })} placeholder="core" /></div>
          <div><Label>Description</Label><Input value={form.module_description} onChange={(e) => setForm({ ...form, module_description: e.target.value })} placeholder="Module description" /></div>
          <div className="flex items-center gap-2"><input type="checkbox" id="mod-enabled" checked={form.module_enabled} onChange={(e) => setForm({ ...form, module_enabled: e.target.checked })} className="h-4 w-4" /><Label htmlFor="mod-enabled" className="cursor-pointer">Enabled</Label></div>
          <Button onClick={createOrUpdate} className="w-full">{editing ? "Save" : "Create"}</Button>
        </div></Dialog>
    </div>
  );
}
