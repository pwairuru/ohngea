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

export function FaxQueue() {
  const [data, setData] = React.useState<any[]>([]);
  const [total, setTotal] = React.useState(0);
  const [q, setQ] = React.useState("");
  const [page, setPage] = React.useState(1);
  const [open, setOpen] = React.useState(false);
  const [editing, setEditing] = React.useState<any | null>(null);
  const [form, setForm] = React.useState({
    fax_uuid: "",
    fax_number: "1001",
    fax_file: "",
    fax_status: "",
    fax_queue_name: "test"
  });

  const load = React.useCallback(() => {
    const params = new URLSearchParams({ page: String(page), per_page: "20" });
    if (q) params.set("q", q);
    apiFetch(`/fax_queue?${params.toString()}`)
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
    fax_uuid: "",
    fax_number: "1001",
    fax_file: "",
    fax_status: "",
    fax_queue_name: "test"
    });
    setOpen(true);
  };

  const startEdit = (row: any) => {
    setEditing(row);
    setForm({
      fax_uuid: row.fax_uuid ?? row["uuid"] ?? "",
      fax_number: row.fax_number ?? row["number"] ?? "",
      fax_file: row.fax_file ?? row["file"] ?? "",
      fax_status: row.fax_status ?? row["status"] ?? "",
      fax_queue_name: row.fax_queue_name ?? row["name"] ?? "",
    });
    setOpen(true);
  };

  const createOrUpdate = async () => {
    try {
      const payload: any = {
        fax_uuid: form.fax_uuid || null,
        fax_number: form.fax_number || null,
        fax_file: form.fax_file || null,
        fax_status: form.fax_status || null,
        fax_queue_name: form.fax_queue_name || null,
        name: form.fax_queue_name || null,
      };
      if (editing) {
        const uid = editing.fax_queue_uuid || editing.id || editing.uuid;
        await apiFetch(`/fax_queue/${uid}`, { method: "PATCH", body: JSON.stringify(payload) });
      } else {
        await apiFetch("/fax_queue", { method: "POST", body: JSON.stringify(payload) });
      }
      setOpen(false);
      setEditing(null);
      load();
    } catch (e: any) {
      alert(e.message);
    }
  };

  const del = async (row: any) => {
    const uid = row.fax_queue_uuid || row.id || row.uuid;
    const label = row.fax_uuid || uid;
    if (!confirm(`Delete ${label}?`)) return;
    try {
      await apiFetch(`/fax_queue/${uid}`, { method: "DELETE" });
      load();
    } catch (e: any) {
      alert(e.message);
    }
  };

  return (
    <div className="p-4 space-y-4">
      <div className="flex justify-between items-center"><h1 className="text-2xl font-bold">Fax Queue</h1><div className="flex gap-2"><Badge>{total} total</Badge><Button onClick={startCreate}>Add Fax Queue</Button></div></div>

      <div className="flex gap-2">
        <Input value={q} onChange={(e) => setQ(e.target.value)} placeholder="Search..." className="flex-1" />
        <Button variant="outline" onClick={() => { setPage(1); load(); }}>Search</Button>
      </div>

      <Card><CardHeader><CardTitle>Fax Queue</CardTitle></CardHeader><CardContent>
          <div className="hidden md:block"><Table><TableHeader><TableRow><TableHead>Fax Uuid</TableHead><TableHead>Fax Number</TableHead><TableHead>Fax File</TableHead><TableHead>Actions</TableHead></TableRow></TableHeader><TableBody>
                {data.map((d: any, i: number) => {
                  const uid = d.fax_queue_uuid || d.id || i;
                  return (
                    <TableRow key={String(uid)}>
                      <TableCell className="text-xs">{d.fax_uuid ?? "-"}</TableCell>
                      <TableCell className="font-mono text-xs">{d.fax_number ?? d["number"] ?? "-"}</TableCell>
                      <TableCell className="text-xs">{d.fax_file ?? "-"}</TableCell>
                      <TableCell><div className="flex gap-1"><Button variant="ghost" aria-label="Edit" title="Edit" className="h-9 w-9 p-0" onClick={() => startEdit(d)}><Pencil className="h-4 w-4" /></Button><Button variant="ghost" aria-label="Delete" title="Delete" className="h-9 w-9 p-0 text-destructive hover:text-destructive hover:bg-destructive/10" onClick={() => del(d)}><Trash2 className="h-4 w-4" /></Button></div></TableCell>
                    </TableRow>
                  );
                })}
                {data.length===0 && <TableRow><TableCell colSpan={4} className="text-center text-muted-foreground py-8">No fax queue — create via Add button</TableCell></TableRow>}
              </TableBody></Table></div>
          <div className="md:hidden grid gap-3">
            {data.map((d: any, i: number) => {
              const uid = d.fax_queue_uuid || d.id || i;
              return (
                <Card key={String(uid)} className="p-3 space-y-1">
                  <div className="flex justify-between items-start"><div className="font-mono font-bold">{d.fax_uuid ?? d["uuid"] ?? String(uid).slice(0,8)}</div></div>
                  <div className="font-mono text-xs">{d.fax_uuid ?? d["uuid"]}</div>
                  <div className="font-mono text-xs">{d.fax_number ?? d["number"]}</div>
                  <div className="text-xs text-muted-foreground truncate">{d.fax_number ?? ""}</div>
                  <div className="flex gap-2 mt-2"><Button variant="ghost" aria-label="Edit" title="Edit" className="flex-1 h-9" onClick={() => startEdit(d)}><Pencil className="h-4 w-4" /></Button><Button variant="ghost" aria-label="Delete" title="Delete" className="flex-1 h-9 text-destructive hover:text-destructive hover:bg-destructive/10" onClick={() => del(d)}><Trash2 className="h-4 w-4" /></Button></div>
                </Card>
              );
            })}
            {data.length===0 && <div className="text-center text-muted-foreground py-4">No fax queue</div>}
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
      <Dialog open={open} onOpenChange={(v) => { setOpen(v); if (!v) setEditing(null); }}><DialogHeader><DialogTitle>{editing ? "Edit Fax Queue" : "Add Fax Queue"}</DialogTitle></DialogHeader>
        <div className="space-y-3">
          <div><Label>Fax Uuid</Label><Input value={form.fax_uuid} onChange={(e) => setForm({ ...form, fax_uuid: e.target.value })} placeholder="fax_uuid" /></div>
          <div><Label>Fax Number</Label><Input value={form.fax_number} onChange={(e) => setForm({ ...form, fax_number: e.target.value })} placeholder="1001" /></div>
          <div><Label>Fax File</Label><Input value={form.fax_file} onChange={(e) => setForm({ ...form, fax_file: e.target.value })} placeholder="fax_file" /></div>
          <div><Label>Fax Status</Label><Input value={form.fax_status} onChange={(e) => setForm({ ...form, fax_status: e.target.value })} placeholder="fax_status" /></div>
          <div><Label>Fax Queue Name</Label><Input value={form.fax_queue_name} onChange={(e) => setForm({ ...form, fax_queue_name: e.target.value })} placeholder="e.g., test" /></div>
          <Button onClick={createOrUpdate} className="w-full">{editing ? "Save" : "Create"}</Button>
        </div></Dialog>
    </div>
  );
}
