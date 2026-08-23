import * as React from "react";
import { apiFetch } from "../lib/api";
import { Card, CardContent, CardHeader, CardTitle } from "../components/ui/card";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "../components/ui/table";
import { Badge } from "../components/ui/badge";
import { Button } from "../components/ui/button";
import { Pencil, Trash2, ChevronsLeft, ChevronLeft, ChevronRight, ChevronsRight } from "lucide-react";
import { Input } from "../components/ui/input";
import { Label } from "../components/ui/label";
import { Dialog, DialogHeader, DialogTitle } from "../components/ui/dialog";

export function CallForward() {
  const [data, setData] = React.useState<any[]>([]);
  const [total, setTotal] = React.useState(0);
  const [q, setQ] = React.useState("");
  const [page, setPage] = React.useState(1);
  const [open, setOpen] = React.useState(false);
  const [editing, setEditing] = React.useState<any | null>(null);
  const [form, setForm] = React.useState({
    extension_uuid: "",
    forward_all_destination: "",
    forward_all_enabled: true,
    forward_busy_destination: "",
    forward_busy_enabled: false,
    forward_no_answer_destination: "",
    forward_no_answer_enabled: false,
    forward_user_not_registered_destination: "",
    forward_user_not_registered_enabled: false,
    enabled: true,
    description: "",
  });

  const load = React.useCallback(() => {
    const params = new URLSearchParams({ page: String(page), per_page: "20" });
    if (q) params.set("q", q);
    apiFetch(`/call_forward?${params.toString()}`)
      .then((r: any) => {
        setData(r.data || []);
        setTotal(r.total ?? r.data?.length ?? 0);
      })
      .catch(() => {});
  }, [page, q]);

  React.useEffect(() => { load(); }, [load]);

  const startCreate = () => {
    setEditing(null);
    setForm({
      extension_uuid: "",
      forward_all_destination: "",
      forward_all_enabled: true,
      forward_busy_destination: "",
      forward_busy_enabled: false,
      forward_no_answer_destination: "",
      forward_no_answer_enabled: false,
      forward_user_not_registered_destination: "",
      forward_user_not_registered_enabled: false,
      enabled: true,
      description: "",
    });
    setOpen(true);
  };

  const startEdit = (row: any) => {
    setEditing(row);
    setForm({
      extension_uuid: row.extension_uuid ?? "",
      forward_all_destination: row.forward_all_destination ?? row.forward_destination ?? "",
      forward_all_enabled: row.forward_all_enabled ?? row.forward_enabled ?? true,
      forward_busy_destination: row.forward_busy_destination ?? "",
      forward_busy_enabled: row.forward_busy_enabled ?? false,
      forward_no_answer_destination: row.forward_no_answer_destination ?? "",
      forward_no_answer_enabled: row.forward_no_answer_enabled ?? false,
      forward_user_not_registered_destination: row.forward_user_not_registered_destination ?? "",
      forward_user_not_registered_enabled: row.forward_user_not_registered_enabled ?? false,
      enabled: row.enabled ?? true,
      description: row.description ?? "",
    });
    setOpen(true);
  };

  const createOrUpdate = async () => {
    try {
      const payload: any = {
        extension_uuid: form.extension_uuid || null,
        forward_all_destination: form.forward_all_destination || null,
        forward_all_enabled: form.forward_all_enabled,
        forward_busy_destination: form.forward_busy_destination || null,
        forward_busy_enabled: form.forward_busy_enabled,
        forward_no_answer_destination: form.forward_no_answer_destination || null,
        forward_no_answer_enabled: form.forward_no_answer_enabled,
        forward_user_not_registered_destination: form.forward_user_not_registered_destination || null,
        forward_user_not_registered_enabled: form.forward_user_not_registered_enabled,
        enabled: form.enabled,
        description: form.description || null,
      };
      if (editing) {
        const uid = editing.call_forward_uuid || editing.id || editing.uuid;
        await apiFetch(`/call_forward/${uid}`, { method: "PATCH", body: JSON.stringify(payload) });
      } else {
        await apiFetch("/call_forward", { method: "POST", body: JSON.stringify(payload) });
      }
      setOpen(false);
      setEditing(null);
      load();
    } catch (e: any) {
      alert(e.message);
    }
  };

  const del = async (row: any) => {
    const uid = row.call_forward_uuid || row.id || row.uuid;
    const label = row.forward_all_destination || uid;
    if (!confirm(`Delete ${label}?`)) return;
    try {
      await apiFetch(`/call_forward/${uid}`, { method: "DELETE" });
      load();
    } catch (e: any) {
      alert(e.message);
    }
  };

  return (
    <div className="p-4 space-y-4">
      <div className="flex justify-between items-center"><h1 className="text-2xl font-bold">Call Forward</h1><div className="flex gap-2"><Badge>{total} total</Badge><Button onClick={startCreate}>Add Forward</Button></div></div>

      <div className="flex gap-2">
        <Input value={q} onChange={(e) => setQ(e.target.value)} placeholder="Search..." className="flex-1" />
        <Button variant="outline" onClick={() => { setPage(1); load(); }}>Search</Button>
      </div>

      <Card><CardHeader><CardTitle>Call Forward</CardTitle></CardHeader><CardContent>
          <div className="hidden md:block"><Table><TableHeader><TableRow><TableHead>Extension</TableHead><TableHead>Forward All</TableHead><TableHead>Busy</TableHead><TableHead>No Answer</TableHead><TableHead>Enabled</TableHead><TableHead>Actions</TableHead></TableRow></TableHeader><TableBody>
                {data.map((d: any, i: number) => {
                  const uid = d.call_forward_uuid || d.id || i;
                  return (
                    <TableRow key={String(uid)}>
                      <TableCell className="font-mono text-xs">{d.extension_uuid ? String(d.extension_uuid).slice(0,8) : "-"}</TableCell>
                      <TableCell className="font-mono text-xs">{d.forward_all_enabled ? (d.forward_all_destination || "-") : <span className="text-muted-foreground">disabled</span>}</TableCell>
                      <TableCell className="font-mono text-xs">{d.forward_busy_enabled ? (d.forward_busy_destination || "-") : "-"}</TableCell>
                      <TableCell className="font-mono text-xs">{d.forward_no_answer_enabled ? (d.forward_no_answer_destination || "-") : "-"}</TableCell>
                      <TableCell>{(d.enabled ?? true) ? <Badge>enabled</Badge> : <Badge className="bg-muted text-muted-foreground">disabled</Badge>}</TableCell>
                      <TableCell><div className="flex gap-1"><Button variant="ghost" aria-label="Edit" title="Edit" className="h-9 w-9 p-0" onClick={() => startEdit(d)}><Pencil className="h-4 w-4" /></Button><Button variant="ghost" aria-label="Delete" title="Delete" className="h-9 w-9 p-0 text-destructive hover:text-destructive hover:bg-destructive/10" onClick={() => del(d)}><Trash2 className="h-4 w-4" /></Button></div></TableCell>
                    </TableRow>
                  );
                })}
                {data.length===0 && <TableRow><TableCell colSpan={6} className="text-center text-muted-foreground py-8">No call forwards — create via Add button</TableCell></TableRow>}
              </TableBody></Table></div>
          <div className="md:hidden grid gap-3">
            {data.map((d: any, i: number) => {
              const uid = d.call_forward_uuid || d.id || i;
              return (
                <Card key={String(uid)} className="p-3 space-y-1">
                  <div className="flex justify-between items-start"><div className="font-mono font-bold">{d.forward_all_destination ?? String(uid).slice(0,8)}</div><Badge>{(d.enabled ?? true) ? "enabled" : "disabled"}</Badge></div>
                  <div className="font-mono text-xs">All: {d.forward_all_enabled ? d.forward_all_destination : "disabled"}</div>
                  <div className="text-xs text-muted-foreground truncate">{d.description ?? ""}</div>
                  <div className="flex gap-2 mt-2"><Button variant="ghost" aria-label="Edit" title="Edit" className="flex-1 h-9" onClick={() => startEdit(d)}><Pencil className="h-4 w-4" /></Button><Button variant="ghost" aria-label="Delete" title="Delete" className="flex-1 h-9 text-destructive hover:text-destructive hover:bg-destructive/10" onClick={() => del(d)}><Trash2 className="h-4 w-4" /></Button></div>
                </Card>
              );
            })}
            {data.length===0 && <div className="text-center text-muted-foreground py-4">No call forwards</div>}
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
      <Dialog open={open} onOpenChange={(v) => { setOpen(v); if (!v) setEditing(null); }}><DialogHeader><DialogTitle>{editing ? "Edit Call Forward" : "Add Call Forward"}</DialogTitle></DialogHeader>
        <div className="space-y-3">
          <div><Label>Extension UUID</Label><Input value={form.extension_uuid} onChange={(e) => setForm({ ...form, extension_uuid: e.target.value })} placeholder="extension_uuid (optional)" /></div>
          <div className="flex items-center gap-2"><input type="checkbox" id="forward_all_enabled" checked={form.forward_all_enabled as boolean} onChange={(e) => setForm({ ...form, forward_all_enabled: e.target.checked })} className="h-4 w-4" /><Label htmlFor="forward_all_enabled" className="cursor-pointer">Forward All Enabled</Label></div>
          <div><Label>Forward All Destination</Label><Input value={form.forward_all_destination} onChange={(e) => setForm({ ...form, forward_all_destination: e.target.value })} placeholder="2000" /></div>
          <div className="flex items-center gap-2"><input type="checkbox" id="forward_busy_enabled" checked={form.forward_busy_enabled as boolean} onChange={(e) => setForm({ ...form, forward_busy_enabled: e.target.checked })} className="h-4 w-4" /><Label htmlFor="forward_busy_enabled" className="cursor-pointer">Forward Busy Enabled</Label></div>
          <div><Label>Forward Busy Destination</Label><Input value={form.forward_busy_destination} onChange={(e) => setForm({ ...form, forward_busy_destination: e.target.value })} placeholder="busy dest" /></div>
          <div className="flex items-center gap-2"><input type="checkbox" id="forward_no_answer_enabled" checked={form.forward_no_answer_enabled as boolean} onChange={(e) => setForm({ ...form, forward_no_answer_enabled: e.target.checked })} className="h-4 w-4" /><Label htmlFor="forward_no_answer_enabled" className="cursor-pointer">Forward No Answer Enabled</Label></div>
          <div><Label>Forward No Answer Destination</Label><Input value={form.forward_no_answer_destination} onChange={(e) => setForm({ ...form, forward_no_answer_destination: e.target.value })} placeholder="no-answer dest" /></div>
          <div className="flex items-center gap-2"><input type="checkbox" id="forward_user_not_registered_enabled" checked={form.forward_user_not_registered_enabled as boolean} onChange={(e) => setForm({ ...form, forward_user_not_registered_enabled: e.target.checked })} className="h-4 w-4" /><Label htmlFor="forward_user_not_registered_enabled" className="cursor-pointer">Forward Not Registered Enabled</Label></div>
          <div><Label>Forward Not Registered Destination</Label><Input value={form.forward_user_not_registered_destination} onChange={(e) => setForm({ ...form, forward_user_not_registered_destination: e.target.value })} placeholder="not registered dest" /></div>
          <div className="flex items-center gap-2"><input type="checkbox" id="enabled" checked={form.enabled as boolean} onChange={(e) => setForm({ ...form, enabled: e.target.checked })} className="h-4 w-4" /><Label htmlFor="enabled" className="cursor-pointer">Enabled</Label></div>
          <div><Label>Description</Label><Input value={form.description} onChange={(e) => setForm({ ...form, description: e.target.value })} placeholder="Description" /></div>
          <Button onClick={createOrUpdate} className="w-full">{editing ? "Save" : "Create"}</Button>
        </div></Dialog>
    </div>
  );
}
