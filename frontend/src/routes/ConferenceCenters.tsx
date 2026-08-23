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

export function ConferenceCenters() {
  const [data, setData] = React.useState<any[]>([]);
  const [total, setTotal] = React.useState(0);
  const [q, setQ] = React.useState("");
  const [page, setPage] = React.useState(1);
  const [open, setOpen] = React.useState(false);
  const [editing, setEditing] = React.useState<any | null>(null);
  const [form, setForm] = React.useState({
    conference_center_name: "test",
    conference_center_extension: "1001",
    conference_center_pin_length: "",
    conference_center_greeting: "",
    conference_center_description: "",
    conference_center_enabled: true
  });

  const load = React.useCallback(() => {
    const params = new URLSearchParams({ page: String(page), per_page: "20" });
    if (q) params.set("q", q);
    apiFetch(`/conference_centers?${params.toString()}`)
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
    conference_center_name: "test",
    conference_center_extension: "1001",
    conference_center_pin_length: "",
    conference_center_greeting: "",
    conference_center_description: "",
    conference_center_enabled: true
    });
    setOpen(true);
  };

  const startEdit = (row: any) => {
    setEditing(row);
    setForm({
      conference_center_name: row.conference_center_name ?? row["name"] ?? "",
      conference_center_extension: row.conference_center_extension ?? row["extension"] ?? "",
      conference_center_pin_length: row.conference_center_pin_length != null ? String(row.conference_center_pin_length) : "",
      conference_center_greeting: row.conference_center_greeting ?? row["greeting"] ?? "",
      conference_center_description: row.conference_center_description ?? row["description"] ?? "",
      conference_center_enabled: row.conference_center_enabled ?? row["enabled"] ?? true,
    });
    setOpen(true);
  };

  const createOrUpdate = async () => {
    try {
      const payload: any = {
        conference_center_name: form.conference_center_name || null,
        conference_center_extension: form.conference_center_extension || null,
        conference_center_pin_length: form.conference_center_pin_length ? Number(form.conference_center_pin_length) : null,
        conference_center_greeting: form.conference_center_greeting || null,
        conference_center_description: form.conference_center_description || null,
        conference_center_enabled: form.conference_center_enabled,
        name: form.conference_center_name || null,
        description: form.conference_center_description || null,
        enabled: form.conference_center_enabled || null,
      };
      if (editing) {
        const uid = editing.conference_center_uuid || editing.id || editing.uuid;
        await apiFetch(`/conference_centers/${uid}`, { method: "PATCH", body: JSON.stringify(payload) });
      } else {
        await apiFetch("/conference_centers", { method: "POST", body: JSON.stringify(payload) });
      }
      setOpen(false);
      setEditing(null);
      load();
    } catch (e: any) {
      alert(e.message);
    }
  };

  const del = async (row: any) => {
    const uid = row.conference_center_uuid || row.id || row.uuid;
    const label = row.conference_center_name || uid;
    if (!confirm(`Delete ${label}?`)) return;
    try {
      await apiFetch(`/conference_centers/${uid}`, { method: "DELETE" });
      load();
    } catch (e: any) {
      alert(e.message);
    }
  };

  return (
    <div className="p-4 space-y-4">
      <div className="flex justify-between items-center"><h1 className="text-2xl font-bold">Conference Centers</h1><div className="flex gap-2"><Badge>{total} total</Badge><Button onClick={startCreate}>Add Conference Centers</Button></div></div>

      <div className="flex gap-2">
        <Input value={q} onChange={(e) => setQ(e.target.value)} placeholder="Search..." className="flex-1" />
        <Button variant="outline" onClick={() => { setPage(1); load(); }}>Search</Button>
      </div>

      <Card><CardHeader><CardTitle>Conference Centers</CardTitle></CardHeader><CardContent>
          <div className="hidden md:block"><Table><TableHeader><TableRow><TableHead>Conference Center Name</TableHead><TableHead>Conference Center Extension</TableHead><TableHead>Conference Center Pin Length</TableHead><TableHead>Enabled</TableHead><TableHead>Actions</TableHead></TableRow></TableHeader><TableBody>
                {data.map((d: any, i: number) => {
                  const uid = d.conference_center_uuid || d.id || i;
                  return (
                    <TableRow key={String(uid)}>
                      <TableCell className="font-mono text-xs">{d.conference_center_name ?? d["name"] ?? "-"}</TableCell>
                      <TableCell className="font-mono text-xs">{d.conference_center_extension ?? d["extension"] ?? "-"}</TableCell>
                      <TableCell className="text-xs">{d.conference_center_pin_length ?? "-"}</TableCell>
                      <TableCell>{(d.conference_center_enabled ?? true) ? <Badge>enabled</Badge> : <Badge className="bg-muted text-muted-foreground">disabled</Badge>}</TableCell>
                      <TableCell><div className="flex gap-1"><Button variant="ghost" aria-label="Edit" title="Edit" className="h-9 w-9 p-0" onClick={() => startEdit(d)}><Pencil className="h-4 w-4" /></Button><Button variant="ghost" aria-label="Delete" title="Delete" className="h-9 w-9 p-0 text-destructive hover:text-destructive hover:bg-destructive/10" onClick={() => del(d)}><Trash2 className="h-4 w-4" /></Button></div></TableCell>
                    </TableRow>
                  );
                })}
                {data.length===0 && <TableRow><TableCell colSpan={5} className="text-center text-muted-foreground py-8">No conference centers — create via Add button</TableCell></TableRow>}
              </TableBody></Table></div>
          <div className="md:hidden grid gap-3">
            {data.map((d: any, i: number) => {
              const uid = d.conference_center_uuid || d.id || i;
              return (
                <Card key={String(uid)} className="p-3 space-y-1">
                  <div className="flex justify-between items-start"><div className="font-mono font-bold">{d.conference_center_name ?? d["name"] ?? String(uid).slice(0,8)}</div><Badge>{(d.conference_center_enabled ?? true) ? "enabled" : "disabled"}</Badge></div>
                  <div className="font-mono text-xs">{d.conference_center_name ?? d["name"]}</div>
                  <div className="font-mono text-xs">{d.conference_center_extension ?? d["extension"]}</div>
                  <div className="text-xs text-muted-foreground truncate">{d.conference_center_extension ?? ""}</div>
                  <div className="flex gap-2 mt-2"><Button variant="ghost" aria-label="Edit" title="Edit" className="flex-1 h-9" onClick={() => startEdit(d)}><Pencil className="h-4 w-4" /></Button><Button variant="ghost" aria-label="Delete" title="Delete" className="flex-1 h-9 text-destructive hover:text-destructive hover:bg-destructive/10" onClick={() => del(d)}><Trash2 className="h-4 w-4" /></Button></div>
                </Card>
              );
            })}
            {data.length===0 && <div className="text-center text-muted-foreground py-4">No conference centers</div>}
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
      <Dialog open={open} onOpenChange={(v) => { setOpen(v); if (!v) setEditing(null); }}><DialogHeader><DialogTitle>{editing ? "Edit Conference Centers" : "Add Conference Centers"}</DialogTitle></DialogHeader>
        <div className="space-y-3">
          <div><Label>Conference Center Name *</Label><Input value={form.conference_center_name} onChange={(e) => setForm({ ...form, conference_center_name: e.target.value })} placeholder="e.g., test" /></div>
          <div><Label>Conference Center Extension</Label><Input value={form.conference_center_extension} onChange={(e) => setForm({ ...form, conference_center_extension: e.target.value })} placeholder="1001" /></div>
          <div><Label>Conference Center Pin Length</Label><Input value={form.conference_center_pin_length} onChange={(e) => setForm({ ...form, conference_center_pin_length: e.target.value })} placeholder="conference_center_pin_length" type="number" /></div>
          <div><Label>Conference Center Greeting</Label><Input value={form.conference_center_greeting} onChange={(e) => setForm({ ...form, conference_center_greeting: e.target.value })} placeholder="conference_center_greeting" /></div>
          <div><Label>Conference Center Description</Label><Input value={form.conference_center_description} onChange={(e) => setForm({ ...form, conference_center_description: e.target.value })} placeholder="Description" /></div>
          <div className="flex items-center gap-2"><input type="checkbox" id="conference_center_enabled" checked={form.conference_center_enabled as boolean} onChange={(e) => setForm({ ...form, conference_center_enabled: e.target.checked })} className="h-4 w-4" /><Label htmlFor="conference_center_enabled" className="cursor-pointer">Conference Center Enabled</Label></div>
          <Button onClick={createOrUpdate} className="w-full">{editing ? "Save" : "Create"}</Button>
        </div></Dialog>
    </div>
  );
}
