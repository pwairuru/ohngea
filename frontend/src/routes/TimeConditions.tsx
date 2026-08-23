import * as React from "react";
import { apiFetch } from "../lib/api";
import { Card, CardContent, CardHeader, CardTitle } from "../components/ui/card";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "../components/ui/table";
import { Badge } from "../components/ui/badge";
import { Button } from "../components/ui/button";
import {Pencil, Trash2, ChevronsLeft, ChevronLeft, ChevronRight, ChevronsRight} from "lucide-react";
import { Input } from "../components/ui/input";
import { Label } from "../components/ui/label";
import { Dialog, DialogHeader, DialogTitle } from "../components/ui/dialog";

export function TimeConditions() {
  const [data, setData] = React.useState<any[]>([]);
  const [total, setTotal] = React.useState(0);
  const [q, setQ] = React.useState("");
  const [page, setPage] = React.useState(1);
  const [open, setOpen] = React.useState(false);
  const [editing, setEditing] = React.useState<any | null>(null);
  const [form, setForm] = React.useState({
    name: "business-hours",
    extension: "800",
    condition: "09:00-17:00 Mon-Fri",
    true_dest: "1000",
    false_dest: "voicemail",
    description: "",
    timezone: "",
    enabled: true,
  });

  const load = React.useCallback(() => {
    const params = new URLSearchParams({ page: String(page), per_page: "20" });
    if (q) params.set("q", q);
    apiFetch(`/time_conditions?${params.toString()}`)
      .then((r: any) => {
        setData(r.data || []);
        setTotal(r.total ?? r.data?.length ?? 0);
      })
      .catch(() => {});
  }, [page, q]);

  React.useEffect(() => { load(); }, [load]);

  const startCreate = () => {
    setEditing(null);
    setForm({ name: "business-hours", extension: "800", condition: "09:00-17:00 Mon-Fri", true_dest: "1000", false_dest: "voicemail", description: "", timezone: "", enabled: true });
    setOpen(true);
  };

  const startEdit = (row: any) => {
    setEditing(row);
    setForm({
      name: row.name || row.time_condition_name || "",
      extension: row.extension || row.time_condition_extension || "",
      condition: row.condition || row.time_of_day || row.minute_of_day || "",
      true_dest: row.true_destination || row.true_dest || "",
      false_dest: row.false_destination || row.false_dest || "",
      description: row.description || row.time_condition_description || "",
      timezone: row.timezone || "",
      enabled: row.enabled ?? row.time_condition_enabled ?? true,
    });
    setOpen(true);
  };

  const createOrUpdate = async () => {
    try {
      const payload: any = {
        name: form.name || null,
        extension: form.extension || null,
        condition: form.condition || null,
        true_destination: form.true_dest || null,
        false_destination: form.false_dest || null,
        true_dest: form.true_dest || null,
        false_dest: form.false_dest || null,
        description: form.description || null,
        timezone: form.timezone || null,
        enabled: form.enabled,
      };
      if (editing) {
        const uid = editing.time_condition_uuid || editing.id || editing.uuid;
        await apiFetch(`/time_conditions/${uid}`, { method: "PATCH", body: JSON.stringify(payload) });
      } else {
        await apiFetch("/time_conditions", { method: "POST", body: JSON.stringify(payload) });
      }
      setOpen(false); setEditing(null); load();
    } catch (e: any) { alert(e.message); }
  };

  const del = async (row: any) => {
    const uid = row.time_condition_uuid || row.id || row.uuid;
    const label = row.name || uid;
    if (!confirm(`Delete time condition ${label}?`)) return;
    try { await apiFetch(`/time_conditions/${uid}`, { method: "DELETE" }); load(); } catch (e: any) { alert(e.message); }
  };

  return (
    <div className="p-4 space-y-4">
      <div className="flex justify-between items-center"><h1 className="text-2xl font-bold">Time Conditions</h1><div className="flex gap-2"><Badge>{total} total</Badge><Button onClick={startCreate}>Add Time Condition</Button></div></div>

      <div className="flex gap-2">
        <Input value={q} onChange={(e) => setQ(e.target.value)} placeholder="Search name, extension..." className="flex-1" />
        <Button variant="outline" onClick={() => { setPage(1); load(); }}>Search</Button>
      </div>

      <Card><CardHeader><CardTitle>Time Conditions — calendar routing</CardTitle></CardHeader><CardContent>
          <div className="hidden md:block"><Table><TableHeader><TableRow><TableHead>Name</TableHead><TableHead>Ext</TableHead><TableHead>Condition</TableHead><TableHead>True → False</TableHead><TableHead>Enabled</TableHead><TableHead>Actions</TableHead></TableRow></TableHeader><TableBody>
                {data.map((d: any) => {
                  const uid = d.time_condition_uuid || d.id;
                  const cond = d.condition || d.time_of_day || d.minute_of_day || "-";
                  const tdest = d.true_destination || d.true_dest || "-";
                  const fdest = d.false_destination || d.false_dest || "-";
                  return (
                    <TableRow key={String(uid)}>
                      <TableCell className="font-mono font-bold">{d.name ?? d.time_condition_name ?? "-"}</TableCell>
                      <TableCell className="font-mono text-xs">{d.extension ?? "-"}</TableCell>
                      <TableCell className="font-mono text-xs">{cond}</TableCell>
                      <TableCell className="text-xs">{tdest} → {fdest}</TableCell>
                      <TableCell>{(d.enabled ?? true) ? <Badge>enabled</Badge> : <Badge className="bg-muted text-muted-foreground">disabled</Badge>}</TableCell>
                      <TableCell><div className="flex gap-1"><Button variant="ghost" aria-label="Edit" title="Edit" className="h-9 w-9 p-0" onClick={() => startEdit(d)}><Pencil className="h-4 w-4" /></Button><Button variant="ghost" aria-label="Delete" title="Delete" className="h-9 w-9 p-0 text-destructive hover:text-destructive hover:bg-destructive/10" onClick={() => del(d)}><Trash2 className="h-4 w-4" /></Button></div></TableCell>
                    </TableRow>
                  );
                })}
                {data.length===0 && <TableRow><TableCell colSpan={6} className="text-center text-muted-foreground py-8">No time conditions — create via Add button</TableCell></TableRow>}
              </TableBody></Table></div>
          <div className="md:hidden grid gap-3">
            {data.map((d: any) => {
              const uid = d.time_condition_uuid || d.id;
              const cond = d.condition || "-";
              return (
                <Card key={String(uid)} className="p-3 space-y-1">
                  <div className="flex justify-between"><div className="font-mono font-bold">{d.name ?? d.time_condition_name}</div><Badge>{(d.enabled ?? true) ? "enabled" : "disabled"}</Badge></div>
                  <div className="font-mono text-xs">{cond}</div>
                  <div className="text-xs">{d.true_destination || d.true_dest} → {d.false_destination || d.false_dest}</div>
                  <div className="flex gap-2 mt-2"><Button variant="ghost" aria-label="Edit" title="Edit" className="flex-1 h-9" onClick={() => startEdit(d)}><Pencil className="h-4 w-4" /></Button><Button variant="ghost" aria-label="Delete" title="Delete" className="flex-1 h-9 text-destructive hover:text-destructive hover:bg-destructive/10" onClick={() => del(d)}><Trash2 className="h-4 w-4" /></Button></div>
                </Card>
              );
            })}
            {data.length===0 && <div className="text-center text-muted-foreground py-4">No time conditions</div>}
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
      <Dialog open={open} onOpenChange={(v) => { setOpen(v); if (!v) setEditing(null); }}><DialogHeader><DialogTitle>{editing ? "Edit Time Condition" : "Add Time Condition"}</DialogTitle></DialogHeader>
        <div className="space-y-3">
          <div className="grid grid-cols-2 gap-2">
            <div><Label>Name *</Label><Input value={form.name} onChange={(e) => setForm({ ...form, name: e.target.value })} placeholder="business-hours" /></div>
            <div><Label>Extension *</Label><Input value={form.extension} onChange={(e) => setForm({ ...form, extension: e.target.value })} placeholder="800" /></div>
          </div>
          <div><Label>Condition</Label><Input value={form.condition} onChange={(e) => setForm({ ...form, condition: e.target.value })} placeholder="09:00-17:00 Mon-Fri" /></div>
          <div className="grid grid-cols-2 gap-2">
            <div><Label>True Dest</Label><Input value={form.true_dest} onChange={(e) => setForm({ ...form, true_dest: e.target.value })} placeholder="1000" /></div>
            <div><Label>False Dest</Label><Input value={form.false_dest} onChange={(e) => setForm({ ...form, false_dest: e.target.value })} placeholder="voicemail" /></div>
          </div>
          <div className="grid grid-cols-2 gap-2">
            <div><Label>Description</Label><Input value={form.description} onChange={(e) => setForm({ ...form, description: e.target.value })} placeholder="Business hours routing" /></div>
            <div><Label>Timezone</Label><Input value={form.timezone} onChange={(e) => setForm({ ...form, timezone: e.target.value })} placeholder="America/New_York" /></div>
          </div>
          <div className="flex items-center gap-2"><input type="checkbox" id="tc-enabled" checked={form.enabled} onChange={(e) => setForm({ ...form, enabled: e.target.checked })} className="h-4 w-4" /><Label htmlFor="tc-enabled" className="cursor-pointer">Enabled</Label></div>
          <Button onClick={createOrUpdate} className="w-full">{editing ? "Save" : "Create"}</Button>
        </div></Dialog>
    </div>
  );
}
