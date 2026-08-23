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

export function FollowMe() {
  const [data, setData] = React.useState<any[]>([]);
  const [total, setTotal] = React.useState(0);
  const [q, setQ] = React.useState("");
  const [page, setPage] = React.useState(1);
  const [open, setOpen] = React.useState(false);
  const [editing, setEditing] = React.useState<any | null>(null);
  const [form, setForm] = React.useState({
    follow_me_enabled: true,
    cid_name_prefix: "test",
    cid_number_prefix: "1001",
    call_prompt: "",
    dial_string: "",
    description: "",
    destinations: ""
  });

  const load = React.useCallback(() => {
    const params = new URLSearchParams({ page: String(page), per_page: "20" });
    if (q) params.set("q", q);
    apiFetch(`/follow_me?${params.toString()}`)
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
    follow_me_enabled: true,
    cid_name_prefix: "test",
    cid_number_prefix: "1001",
    call_prompt: "",
    dial_string: "",
    description: "",
    destinations: ""
    });
    setOpen(true);
  };

  const startEdit = (row: any) => {
    setEditing(row);
    setForm({
      follow_me_enabled: row.follow_me_enabled ?? row["enabled"] ?? true,
      cid_name_prefix: row.cid_name_prefix ?? row["prefix"] ?? "",
      cid_number_prefix: row.cid_number_prefix ?? row["prefix"] ?? "",
      call_prompt: row.call_prompt ?? row["prompt"] ?? "",
      dial_string: row.dial_string ?? row["string"] ?? "",
      description: row.description ?? row["description"] ?? "",
      destinations: row.destinations ?? row["destinations"] ?? "",
    });
    setOpen(true);
  };

  const createOrUpdate = async () => {
    try {
      const payload: any = {
        follow_me_enabled: form.follow_me_enabled,
        cid_name_prefix: form.cid_name_prefix || null,
        cid_number_prefix: form.cid_number_prefix || null,
        call_prompt: form.call_prompt || null,
        dial_string: form.dial_string || null,
        description: form.description || null,
        destinations: form.destinations || null,
        enabled: form.follow_me_enabled || null,
      };
      if (editing) {
        const uid = editing.follow_me_uuid || editing.id || editing.uuid;
        await apiFetch(`/follow_me/${uid}`, { method: "PATCH", body: JSON.stringify(payload) });
      } else {
        await apiFetch("/follow_me", { method: "POST", body: JSON.stringify(payload) });
      }
      setOpen(false);
      setEditing(null);
      load();
    } catch (e: any) {
      alert(e.message);
    }
  };

  const del = async (row: any) => {
    const uid = row.follow_me_uuid || row.id || row.uuid;
    const label = row.cid_name_prefix || uid;
    if (!confirm(`Delete ${label}?`)) return;
    try {
      await apiFetch(`/follow_me/${uid}`, { method: "DELETE" });
      load();
    } catch (e: any) {
      alert(e.message);
    }
  };

  return (
    <div className="p-4 space-y-4">
      <div className="flex justify-between items-center"><h1 className="text-2xl font-bold">Follow Me</h1><div className="flex gap-2"><Badge>{total} total</Badge><Button onClick={startCreate}>Add Follow Me</Button></div></div>

      <div className="flex gap-2">
        <Input value={q} onChange={(e) => setQ(e.target.value)} placeholder="Search..." className="flex-1" />
        <Button variant="outline" onClick={() => { setPage(1); load(); }}>Search</Button>
      </div>

      <Card><CardHeader><CardTitle>Follow Me</CardTitle></CardHeader><CardContent>
          <div className="hidden md:block"><Table><TableHeader><TableRow><TableHead>Cid Name Prefix</TableHead><TableHead>Cid Number Prefix</TableHead><TableHead>Call Prompt</TableHead><TableHead>Enabled</TableHead><TableHead>Actions</TableHead></TableRow></TableHeader><TableBody>
                {data.map((d: any, i: number) => {
                  const uid = d.follow_me_uuid || d.id || i;
                  return (
                    <TableRow key={String(uid)}>
                      <TableCell className="font-mono text-xs">{d.cid_name_prefix ?? d["prefix"] ?? "-"}</TableCell>
                      <TableCell className="font-mono text-xs">{d.cid_number_prefix ?? d["prefix"] ?? "-"}</TableCell>
                      <TableCell className="text-xs">{d.call_prompt ?? "-"}</TableCell>
                      <TableCell>{(d.follow_me_enabled ?? true) ? <Badge>enabled</Badge> : <Badge className="bg-muted text-muted-foreground">disabled</Badge>}</TableCell>
                      <TableCell><div className="flex gap-1"><Button variant="ghost" aria-label="Edit" title="Edit" className="h-9 w-9 p-0" onClick={() => startEdit(d)}><Pencil className="h-4 w-4" /></Button><Button variant="ghost" aria-label="Delete" title="Delete" className="h-9 w-9 p-0 text-destructive hover:text-destructive hover:bg-destructive/10" onClick={() => del(d)}><Trash2 className="h-4 w-4" /></Button></div></TableCell>
                    </TableRow>
                  );
                })}
                {data.length===0 && <TableRow><TableCell colSpan={5} className="text-center text-muted-foreground py-8">No follow me — create via Add button</TableCell></TableRow>}
              </TableBody></Table></div>
          <div className="md:hidden grid gap-3">
            {data.map((d: any, i: number) => {
              const uid = d.follow_me_uuid || d.id || i;
              return (
                <Card key={String(uid)} className="p-3 space-y-1">
                  <div className="flex justify-between items-start"><div className="font-mono font-bold">{d.cid_name_prefix ?? d["prefix"] ?? String(uid).slice(0,8)}</div><Badge>{(d.follow_me_enabled ?? true) ? "enabled" : "disabled"}</Badge></div>
                  <div className="font-mono text-xs">{d.cid_name_prefix ?? d["prefix"]}</div>
                  <div className="font-mono text-xs">{d.cid_number_prefix ?? d["prefix"]}</div>
                  <div className="text-xs text-muted-foreground truncate">{d.cid_number_prefix ?? ""}</div>
                  <div className="flex gap-2 mt-2"><Button variant="ghost" aria-label="Edit" title="Edit" className="flex-1 h-9" onClick={() => startEdit(d)}><Pencil className="h-4 w-4" /></Button><Button variant="ghost" aria-label="Delete" title="Delete" className="flex-1 h-9 text-destructive hover:text-destructive hover:bg-destructive/10" onClick={() => del(d)}><Trash2 className="h-4 w-4" /></Button></div>
                </Card>
              );
            })}
            {data.length===0 && <div className="text-center text-muted-foreground py-4">No follow me</div>}
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
      <Dialog open={open} onOpenChange={(v) => { setOpen(v); if (!v) setEditing(null); }}><DialogHeader><DialogTitle>{editing ? "Edit Follow Me" : "Add Follow Me"}</DialogTitle></DialogHeader>
        <div className="space-y-3">
          <div className="flex items-center gap-2"><input type="checkbox" id="follow_me_enabled" checked={form.follow_me_enabled as boolean} onChange={(e) => setForm({ ...form, follow_me_enabled: e.target.checked })} className="h-4 w-4" /><Label htmlFor="follow_me_enabled" className="cursor-pointer">Follow Me Enabled</Label></div>
          <div><Label>Cid Name Prefix *</Label><Input value={form.cid_name_prefix} onChange={(e) => setForm({ ...form, cid_name_prefix: e.target.value })} placeholder="e.g., test" /></div>
          <div><Label>Cid Number Prefix</Label><Input value={form.cid_number_prefix} onChange={(e) => setForm({ ...form, cid_number_prefix: e.target.value })} placeholder="1001" /></div>
          <div><Label>Call Prompt</Label><Input value={form.call_prompt} onChange={(e) => setForm({ ...form, call_prompt: e.target.value })} placeholder="call_prompt" /></div>
          <div><Label>Dial String</Label><Input value={form.dial_string} onChange={(e) => setForm({ ...form, dial_string: e.target.value })} placeholder="dial_string" /></div>
          <div><Label>Description</Label><Input value={form.description} onChange={(e) => setForm({ ...form, description: e.target.value })} placeholder="Description" /></div>
          <div><Label>Destinations</Label><Input value={form.destinations} onChange={(e) => setForm({ ...form, destinations: e.target.value })} placeholder="destinations" /></div>
          <Button onClick={createOrUpdate} className="w-full">{editing ? "Save" : "Create"}</Button>
        </div></Dialog>
    </div>
  );
}
