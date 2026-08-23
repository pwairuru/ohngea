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

export function CallBroadcasts() {
  const [data, setData] = React.useState<any[]>([]);
  const [total, setTotal] = React.useState(0);
  const [q, setQ] = React.useState("");
  const [page, setPage] = React.useState(1);
  const [open, setOpen] = React.useState(false);
  const [editing, setEditing] = React.useState<any | null>(null);
  const [form, setForm] = React.useState({
    broadcast_name: "test",
    broadcast_description: "",
    broadcast_timeout: "",
    broadcast_concurrent_limit: "",
    broadcast_caller_id_name: "test",
    broadcast_caller_id_number: "1001",
    broadcast_destination_type: "",
    broadcast_phone_numbers: "1001"
  });

  const load = React.useCallback(() => {
    const params = new URLSearchParams({ page: String(page), per_page: "20" });
    if (q) params.set("q", q);
    apiFetch(`/call_broadcasts?${params.toString()}`)
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
    broadcast_name: "test",
    broadcast_description: "",
    broadcast_timeout: "",
    broadcast_concurrent_limit: "",
    broadcast_caller_id_name: "test",
    broadcast_caller_id_number: "1001",
    broadcast_destination_type: "",
    broadcast_phone_numbers: "1001"
    });
    setOpen(true);
  };

  const startEdit = (row: any) => {
    setEditing(row);
    setForm({
      broadcast_name: row.broadcast_name ?? row["name"] ?? "",
      broadcast_description: row.broadcast_description ?? row["description"] ?? "",
      broadcast_timeout: row.broadcast_timeout != null ? String(row.broadcast_timeout) : "",
      broadcast_concurrent_limit: row.broadcast_concurrent_limit != null ? String(row.broadcast_concurrent_limit) : "",
      broadcast_caller_id_name: row.broadcast_caller_id_name ?? row["name"] ?? "",
      broadcast_caller_id_number: row.broadcast_caller_id_number ?? row["number"] ?? "",
      broadcast_destination_type: row.broadcast_destination_type ?? row["type"] ?? "",
      broadcast_phone_numbers: row.broadcast_phone_numbers ?? row["numbers"] ?? "",
    });
    setOpen(true);
  };

  const createOrUpdate = async () => {
    try {
      const payload: any = {
        broadcast_name: form.broadcast_name || null,
        broadcast_description: form.broadcast_description || null,
        broadcast_timeout: form.broadcast_timeout ? Number(form.broadcast_timeout) : null,
        broadcast_concurrent_limit: form.broadcast_concurrent_limit ? Number(form.broadcast_concurrent_limit) : null,
        broadcast_caller_id_name: form.broadcast_caller_id_name || null,
        broadcast_caller_id_number: form.broadcast_caller_id_number || null,
        broadcast_destination_type: form.broadcast_destination_type || null,
        broadcast_phone_numbers: form.broadcast_phone_numbers || null,
        name: form.broadcast_name || null,
        description: form.broadcast_description || null,
      };
      if (editing) {
        const uid = editing.call_broadcast_uuid || editing.id || editing.uuid;
        await apiFetch(`/call_broadcasts/${uid}`, { method: "PATCH", body: JSON.stringify(payload) });
      } else {
        await apiFetch("/call_broadcasts", { method: "POST", body: JSON.stringify(payload) });
      }
      setOpen(false);
      setEditing(null);
      load();
    } catch (e: any) {
      alert(e.message);
    }
  };

  const del = async (row: any) => {
    const uid = row.call_broadcast_uuid || row.id || row.uuid;
    const label = row.broadcast_name || uid;
    if (!confirm(`Delete ${label}?`)) return;
    try {
      await apiFetch(`/call_broadcasts/${uid}`, { method: "DELETE" });
      load();
    } catch (e: any) {
      alert(e.message);
    }
  };

  return (
    <div className="p-4 space-y-4">
      <div className="flex justify-between items-center"><h1 className="text-2xl font-bold">Call Broadcasts</h1><div className="flex gap-2"><Badge>{total} total</Badge><Button onClick={startCreate}>Add Call Broadcasts</Button></div></div>

      <div className="flex gap-2">
        <Input value={q} onChange={(e) => setQ(e.target.value)} placeholder="Search..." className="flex-1" />
        <Button variant="outline" onClick={() => { setPage(1); load(); }}>Search</Button>
      </div>

      <Card><CardHeader><CardTitle>Call Broadcasts</CardTitle></CardHeader><CardContent>
          <div className="hidden md:block"><Table><TableHeader><TableRow><TableHead>Broadcast Name</TableHead><TableHead>Broadcast Description</TableHead><TableHead>Broadcast Timeout</TableHead><TableHead>Actions</TableHead></TableRow></TableHeader><TableBody>
                {data.map((d: any, i: number) => {
                  const uid = d.call_broadcast_uuid || d.id || i;
                  return (
                    <TableRow key={String(uid)}>
                      <TableCell className="font-mono text-xs">{d.broadcast_name ?? d["name"] ?? "-"}</TableCell>
                      <TableCell className="text-xs text-muted-foreground max-w-[200px] truncate" title={d.broadcast_description || ""}>{d.broadcast_description ?? "-"}</TableCell>
                      <TableCell className="text-xs">{d.broadcast_timeout ?? "-"}</TableCell>
                      <TableCell><div className="flex gap-1"><Button variant="ghost" aria-label="Edit" title="Edit" className="h-9 w-9 p-0" onClick={() => startEdit(d)}><Pencil className="h-4 w-4" /></Button><Button variant="ghost" aria-label="Delete" title="Delete" className="h-9 w-9 p-0 text-destructive hover:text-destructive hover:bg-destructive/10" onClick={() => del(d)}><Trash2 className="h-4 w-4" /></Button></div></TableCell>
                    </TableRow>
                  );
                })}
                {data.length===0 && <TableRow><TableCell colSpan={4} className="text-center text-muted-foreground py-8">No call broadcasts — create via Add button</TableCell></TableRow>}
              </TableBody></Table></div>
          <div className="md:hidden grid gap-3">
            {data.map((d: any, i: number) => {
              const uid = d.call_broadcast_uuid || d.id || i;
              return (
                <Card key={String(uid)} className="p-3 space-y-1">
                  <div className="flex justify-between items-start"><div className="font-mono font-bold">{d.broadcast_name ?? d["name"] ?? String(uid).slice(0,8)}</div></div>
                  <div className="font-mono text-xs">{d.broadcast_name ?? d["name"]}</div>
                  <div className="font-mono text-xs">{d.broadcast_description ?? d["description"]}</div>
                  <div className="text-xs text-muted-foreground truncate">{d.broadcast_description ?? ""}</div>
                  <div className="flex gap-2 mt-2"><Button variant="ghost" aria-label="Edit" title="Edit" className="flex-1 h-9" onClick={() => startEdit(d)}><Pencil className="h-4 w-4" /></Button><Button variant="ghost" aria-label="Delete" title="Delete" className="flex-1 h-9 text-destructive hover:text-destructive hover:bg-destructive/10" onClick={() => del(d)}><Trash2 className="h-4 w-4" /></Button></div>
                </Card>
              );
            })}
            {data.length===0 && <div className="text-center text-muted-foreground py-4">No call broadcasts</div>}
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
      <Dialog open={open} onOpenChange={(v) => { setOpen(v); if (!v) setEditing(null); }}><DialogHeader><DialogTitle>{editing ? "Edit Call Broadcasts" : "Add Call Broadcasts"}</DialogTitle></DialogHeader>
        <div className="space-y-3">
          <div><Label>Broadcast Name *</Label><Input value={form.broadcast_name} onChange={(e) => setForm({ ...form, broadcast_name: e.target.value })} placeholder="e.g., test" /></div>
          <div><Label>Broadcast Description</Label><Input value={form.broadcast_description} onChange={(e) => setForm({ ...form, broadcast_description: e.target.value })} placeholder="Description" /></div>
          <div><Label>Broadcast Timeout</Label><Input value={form.broadcast_timeout} onChange={(e) => setForm({ ...form, broadcast_timeout: e.target.value })} placeholder="broadcast_timeout" type="number" /></div>
          <div><Label>Broadcast Concurrent Limit</Label><Input value={form.broadcast_concurrent_limit} onChange={(e) => setForm({ ...form, broadcast_concurrent_limit: e.target.value })} placeholder="broadcast_concurrent_limit" type="number" /></div>
          <div><Label>Broadcast Caller Id Name</Label><Input value={form.broadcast_caller_id_name} onChange={(e) => setForm({ ...form, broadcast_caller_id_name: e.target.value })} placeholder="e.g., test" /></div>
          <div><Label>Broadcast Caller Id Number</Label><Input value={form.broadcast_caller_id_number} onChange={(e) => setForm({ ...form, broadcast_caller_id_number: e.target.value })} placeholder="1001" /></div>
          <div><Label>Broadcast Destination Type</Label><Input value={form.broadcast_destination_type} onChange={(e) => setForm({ ...form, broadcast_destination_type: e.target.value })} placeholder="broadcast_destination_type" /></div>
          <div><Label>Broadcast Phone Numbers</Label><Input value={form.broadcast_phone_numbers} onChange={(e) => setForm({ ...form, broadcast_phone_numbers: e.target.value })} placeholder="1001" /></div>
          <Button onClick={createOrUpdate} className="w-full">{editing ? "Save" : "Create"}</Button>
        </div></Dialog>
    </div>
  );
}
