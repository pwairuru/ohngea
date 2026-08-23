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

export function EmailQueue() {
  const [data, setData] = React.useState<any[]>([]);
  const [total, setTotal] = React.useState(0);
  const [q, setQ] = React.useState("");
  const [page, setPage] = React.useState(1);
  const [open, setOpen] = React.useState(false);
  const [editing, setEditing] = React.useState<any | null>(null);
  const [form, setForm] = React.useState({
    email_from: "",
    email_to: "",
    email_subject: "",
    email_body: "",
    email_status: ""
  });

  const load = React.useCallback(() => {
    const params = new URLSearchParams({ page: String(page), per_page: "20" });
    if (q) params.set("q", q);
    apiFetch(`/email_queue?${params.toString()}`)
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
    email_from: "",
    email_to: "",
    email_subject: "",
    email_body: "",
    email_status: ""
    });
    setOpen(true);
  };

  const startEdit = (row: any) => {
    setEditing(row);
    setForm({
      email_from: row.email_from ?? row["from"] ?? "",
      email_to: row.email_to ?? row["to"] ?? "",
      email_subject: row.email_subject ?? row["subject"] ?? "",
      email_body: row.email_body ?? row["body"] ?? "",
      email_status: row.email_status ?? row["status"] ?? "",
    });
    setOpen(true);
  };

  const createOrUpdate = async () => {
    try {
      const payload: any = {
        email_from: form.email_from || null,
        email_to: form.email_to || null,
        email_subject: form.email_subject || null,
        email_body: form.email_body || null,
        email_status: form.email_status || null,
      };
      if (editing) {
        const uid = editing.email_queue_uuid || editing.id || editing.uuid;
        await apiFetch(`/email_queue/${uid}`, { method: "PATCH", body: JSON.stringify(payload) });
      } else {
        await apiFetch("/email_queue", { method: "POST", body: JSON.stringify(payload) });
      }
      setOpen(false);
      setEditing(null);
      load();
    } catch (e: any) {
      alert(e.message);
    }
  };

  const del = async (row: any) => {
    const uid = row.email_queue_uuid || row.id || row.uuid;
    const label = row.email_from || uid;
    if (!confirm(`Delete ${label}?`)) return;
    try {
      await apiFetch(`/email_queue/${uid}`, { method: "DELETE" });
      load();
    } catch (e: any) {
      alert(e.message);
    }
  };

  return (
    <div className="p-4 space-y-4">
      <div className="flex justify-between items-center"><h1 className="text-2xl font-bold">Email Queue</h1><div className="flex gap-2"><Badge>{total} total</Badge><Button onClick={startCreate}>Add Email Queue</Button></div></div>

      <div className="flex gap-2">
        <Input value={q} onChange={(e) => setQ(e.target.value)} placeholder="Search..." className="flex-1" />
        <Button variant="outline" onClick={() => { setPage(1); load(); }}>Search</Button>
      </div>

      <Card><CardHeader><CardTitle>Email Queue</CardTitle></CardHeader><CardContent>
          <div className="hidden md:block"><Table><TableHeader><TableRow><TableHead>Email From</TableHead><TableHead>Email To</TableHead><TableHead>Email Subject</TableHead><TableHead>Actions</TableHead></TableRow></TableHeader><TableBody>
                {data.map((d: any, i: number) => {
                  const uid = d.email_queue_uuid || d.id || i;
                  return (
                    <TableRow key={String(uid)}>
                      <TableCell className="text-xs">{d.email_from ?? "-"}</TableCell>
                      <TableCell className="text-xs">{d.email_to ?? "-"}</TableCell>
                      <TableCell className="text-xs">{d.email_subject ?? "-"}</TableCell>
                      <TableCell><div className="flex gap-1"><Button variant="ghost" aria-label="Edit" title="Edit" className="h-9 w-9 p-0" onClick={() => startEdit(d)}><Pencil className="h-4 w-4" /></Button><Button variant="ghost" aria-label="Delete" title="Delete" className="h-9 w-9 p-0 text-destructive hover:text-destructive hover:bg-destructive/10" onClick={() => del(d)}><Trash2 className="h-4 w-4" /></Button></div></TableCell>
                    </TableRow>
                  );
                })}
                {data.length===0 && <TableRow><TableCell colSpan={4} className="text-center text-muted-foreground py-8">No email queue — create via Add button</TableCell></TableRow>}
              </TableBody></Table></div>
          <div className="md:hidden grid gap-3">
            {data.map((d: any, i: number) => {
              const uid = d.email_queue_uuid || d.id || i;
              return (
                <Card key={String(uid)} className="p-3 space-y-1">
                  <div className="flex justify-between items-start"><div className="font-mono font-bold">{d.email_from ?? d["from"] ?? String(uid).slice(0,8)}</div></div>
                  <div className="font-mono text-xs">{d.email_from ?? d["from"]}</div>
                  <div className="font-mono text-xs">{d.email_to ?? d["to"]}</div>
                  <div className="text-xs text-muted-foreground truncate">{d.email_to ?? ""}</div>
                  <div className="flex gap-2 mt-2"><Button variant="ghost" aria-label="Edit" title="Edit" className="flex-1 h-9" onClick={() => startEdit(d)}><Pencil className="h-4 w-4" /></Button><Button variant="ghost" aria-label="Delete" title="Delete" className="flex-1 h-9 text-destructive hover:text-destructive hover:bg-destructive/10" onClick={() => del(d)}><Trash2 className="h-4 w-4" /></Button></div>
                </Card>
              );
            })}
            {data.length===0 && <div className="text-center text-muted-foreground py-4">No email queue</div>}
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
      <Dialog open={open} onOpenChange={(v) => { setOpen(v); if (!v) setEditing(null); }}><DialogHeader><DialogTitle>{editing ? "Edit Email Queue" : "Add Email Queue"}</DialogTitle></DialogHeader>
        <div className="space-y-3">
          <div><Label>Email From</Label><Input value={form.email_from} onChange={(e) => setForm({ ...form, email_from: e.target.value })} placeholder="email_from" /></div>
          <div><Label>Email To</Label><Input value={form.email_to} onChange={(e) => setForm({ ...form, email_to: e.target.value })} placeholder="email_to" /></div>
          <div><Label>Email Subject</Label><Input value={form.email_subject} onChange={(e) => setForm({ ...form, email_subject: e.target.value })} placeholder="email_subject" /></div>
          <div><Label>Email Body</Label><Input value={form.email_body} onChange={(e) => setForm({ ...form, email_body: e.target.value })} placeholder="email_body" /></div>
          <div><Label>Email Status</Label><Input value={form.email_status} onChange={(e) => setForm({ ...form, email_status: e.target.value })} placeholder="email_status" /></div>
          <Button onClick={createOrUpdate} className="w-full">{editing ? "Save" : "Create"}</Button>
        </div></Dialog>
    </div>
  );
}
