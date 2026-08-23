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

export function MusicOnHold() {
  const [data, setData] = React.useState<any[]>([]);
  const [total, setTotal] = React.useState(0);
  const [q, setQ] = React.useState("");
  const [page, setPage] = React.useState(1);
  const [open, setOpen] = React.useState(false);
  const [editing, setEditing] = React.useState<any | null>(null);
  const [form, setForm] = React.useState({
    music_name: "test",
    music_path: "",
    music_rate: "",
    music_shuffle: "",
    enabled: true,
    description: ""
  });

  const load = React.useCallback(() => {
    const params = new URLSearchParams({ page: String(page), per_page: "20" });
    if (q) params.set("q", q);
    apiFetch(`/music_on_hold?${params.toString()}`)
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
    music_name: "test",
    music_path: "",
    music_rate: "",
    music_shuffle: "",
    enabled: true,
    description: ""
    });
    setOpen(true);
  };

  const startEdit = (row: any) => {
    setEditing(row);
    setForm({
      music_name: row.music_name ?? row["name"] ?? "",
      music_path: row.music_path ?? row["path"] ?? "",
      music_rate: row.music_rate != null ? String(row.music_rate) : "",
      music_shuffle: row.music_shuffle ?? row["shuffle"] ?? "",
      enabled: row.enabled ?? row["enabled"] ?? true,
      description: row.description ?? row["description"] ?? "",
    });
    setOpen(true);
  };

  const createOrUpdate = async () => {
    try {
      const payload: any = {
        music_name: form.music_name || null,
        music_path: form.music_path || null,
        music_rate: form.music_rate ? Number(form.music_rate) : null,
        music_shuffle: form.music_shuffle || null,
        enabled: form.enabled,
        description: form.description || null,
        name: form.music_name || null,
      };
      if (editing) {
        const uid = editing.music_on_hold_uuid || editing.id || editing.uuid;
        await apiFetch(`/music_on_hold/${uid}`, { method: "PATCH", body: JSON.stringify(payload) });
      } else {
        await apiFetch("/music_on_hold", { method: "POST", body: JSON.stringify(payload) });
      }
      setOpen(false);
      setEditing(null);
      load();
    } catch (e: any) {
      alert(e.message);
    }
  };

  const del = async (row: any) => {
    const uid = row.music_on_hold_uuid || row.id || row.uuid;
    const label = row.music_name || uid;
    if (!confirm(`Delete ${label}?`)) return;
    try {
      await apiFetch(`/music_on_hold/${uid}`, { method: "DELETE" });
      load();
    } catch (e: any) {
      alert(e.message);
    }
  };

  return (
    <div className="p-4 space-y-4">
      <div className="flex justify-between items-center"><h1 className="text-2xl font-bold">Music On Hold</h1><div className="flex gap-2"><Badge>{total} total</Badge><Button onClick={startCreate}>Add Music On Hold</Button></div></div>

      <div className="flex gap-2">
        <Input value={q} onChange={(e) => setQ(e.target.value)} placeholder="Search..." className="flex-1" />
        <Button variant="outline" onClick={() => { setPage(1); load(); }}>Search</Button>
      </div>

      <Card><CardHeader><CardTitle>Music On Hold</CardTitle></CardHeader><CardContent>
          <div className="hidden md:block"><Table><TableHeader><TableRow><TableHead>Music Name</TableHead><TableHead>Music Path</TableHead><TableHead>Music Rate</TableHead><TableHead>Enabled</TableHead><TableHead>Actions</TableHead></TableRow></TableHeader><TableBody>
                {data.map((d: any, i: number) => {
                  const uid = d.music_on_hold_uuid || d.id || i;
                  return (
                    <TableRow key={String(uid)}>
                      <TableCell className="font-mono text-xs">{d.music_name ?? d["name"] ?? "-"}</TableCell>
                      <TableCell className="text-xs">{d.music_path ?? "-"}</TableCell>
                      <TableCell className="text-xs">{d.music_rate ?? "-"}</TableCell>
                      <TableCell>{(d.enabled ?? true) ? <Badge>enabled</Badge> : <Badge className="bg-muted text-muted-foreground">disabled</Badge>}</TableCell>
                      <TableCell><div className="flex gap-1"><Button variant="ghost" aria-label="Edit" title="Edit" className="h-9 w-9 p-0" onClick={() => startEdit(d)}><Pencil className="h-4 w-4" /></Button><Button variant="ghost" aria-label="Delete" title="Delete" className="h-9 w-9 p-0 text-destructive hover:text-destructive hover:bg-destructive/10" onClick={() => del(d)}><Trash2 className="h-4 w-4" /></Button></div></TableCell>
                    </TableRow>
                  );
                })}
                {data.length===0 && <TableRow><TableCell colSpan={5} className="text-center text-muted-foreground py-8">No music on hold — create via Add button</TableCell></TableRow>}
              </TableBody></Table></div>
          <div className="md:hidden grid gap-3">
            {data.map((d: any, i: number) => {
              const uid = d.music_on_hold_uuid || d.id || i;
              return (
                <Card key={String(uid)} className="p-3 space-y-1">
                  <div className="flex justify-between items-start"><div className="font-mono font-bold">{d.music_name ?? d["name"] ?? String(uid).slice(0,8)}</div><Badge>{(d.enabled ?? true) ? "enabled" : "disabled"}</Badge></div>
                  <div className="font-mono text-xs">{d.music_name ?? d["name"]}</div>
                  <div className="font-mono text-xs">{d.music_path ?? d["path"]}</div>
                  <div className="text-xs text-muted-foreground truncate">{d.music_path ?? ""}</div>
                  <div className="flex gap-2 mt-2"><Button variant="ghost" aria-label="Edit" title="Edit" className="flex-1 h-9" onClick={() => startEdit(d)}><Pencil className="h-4 w-4" /></Button><Button variant="ghost" aria-label="Delete" title="Delete" className="flex-1 h-9 text-destructive hover:text-destructive hover:bg-destructive/10" onClick={() => del(d)}><Trash2 className="h-4 w-4" /></Button></div>
                </Card>
              );
            })}
            {data.length===0 && <div className="text-center text-muted-foreground py-4">No music on hold</div>}
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
      <Dialog open={open} onOpenChange={(v) => { setOpen(v); if (!v) setEditing(null); }}><DialogHeader><DialogTitle>{editing ? "Edit Music On Hold" : "Add Music On Hold"}</DialogTitle></DialogHeader>
        <div className="space-y-3">
          <div><Label>Music Name *</Label><Input value={form.music_name} onChange={(e) => setForm({ ...form, music_name: e.target.value })} placeholder="e.g., test" /></div>
          <div><Label>Music Path</Label><Input value={form.music_path} onChange={(e) => setForm({ ...form, music_path: e.target.value })} placeholder="music_path" /></div>
          <div><Label>Music Rate</Label><Input value={form.music_rate} onChange={(e) => setForm({ ...form, music_rate: e.target.value })} placeholder="music_rate" type="number" /></div>
          <div><Label>Music Shuffle</Label><Input value={form.music_shuffle} onChange={(e) => setForm({ ...form, music_shuffle: e.target.value })} placeholder="music_shuffle" /></div>
          <div className="flex items-center gap-2"><input type="checkbox" id="enabled" checked={form.enabled as boolean} onChange={(e) => setForm({ ...form, enabled: e.target.checked })} className="h-4 w-4" /><Label htmlFor="enabled" className="cursor-pointer">Enabled</Label></div>
          <div><Label>Description</Label><Input value={form.description} onChange={(e) => setForm({ ...form, description: e.target.value })} placeholder="Description" /></div>
          <Button onClick={createOrUpdate} className="w-full">{editing ? "Save" : "Create"}</Button>
        </div></Dialog>
    </div>
  );
}
