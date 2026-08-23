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

export function Vars() {
  const [data, setData] = React.useState<any[]>([]);
  const [total, setTotal] = React.useState(0);
  const [q, setQ] = React.useState("");
  const [page, setPage] = React.useState(1);
  const [open, setOpen] = React.useState(false);
  const [editing, setEditing] = React.useState<any | null>(null);
  const [form, setForm] = React.useState({
    var_category: "",
    var_name: "test",
    var_value: "",
    var_enabled: true,
    var_description: ""
  });

  const load = React.useCallback(() => {
    const params = new URLSearchParams({ page: String(page), per_page: "20" });
    if (q) params.set("q", q);
    apiFetch(`/vars?${params.toString()}`)
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
    var_category: "",
    var_name: "test",
    var_value: "",
    var_enabled: true,
    var_description: ""
    });
    setOpen(true);
  };

  const startEdit = (row: any) => {
    setEditing(row);
    setForm({
      var_category: row.var_category ?? row["category"] ?? "",
      var_name: row.var_name ?? row["name"] ?? "",
      var_value: row.var_value ?? row["value"] ?? "",
      var_enabled: row.var_enabled ?? row["enabled"] ?? true,
      var_description: row.var_description ?? row["description"] ?? "",
    });
    setOpen(true);
  };

  const createOrUpdate = async () => {
    try {
      const payload: any = {
        var_category: form.var_category || null,
        var_name: form.var_name || null,
        var_value: form.var_value || null,
        var_enabled: form.var_enabled,
        var_description: form.var_description || null,
        category: form.var_category || null,
        name: form.var_name || null,
        value: form.var_value || null,
        enabled: form.var_enabled || null,
        description: form.var_description || null,
      };
      if (editing) {
        const uid = editing.var_uuid || editing.id || editing.uuid;
        await apiFetch(`/vars/${uid}`, { method: "PATCH", body: JSON.stringify(payload) });
      } else {
        await apiFetch("/vars", { method: "POST", body: JSON.stringify(payload) });
      }
      setOpen(false);
      setEditing(null);
      load();
    } catch (e: any) {
      alert(e.message);
    }
  };

  const del = async (row: any) => {
    const uid = row.var_uuid || row.id || row.uuid;
    const label = row.var_category || uid;
    if (!confirm(`Delete ${label}?`)) return;
    try {
      await apiFetch(`/vars/${uid}`, { method: "DELETE" });
      load();
    } catch (e: any) {
      alert(e.message);
    }
  };

  return (
    <div className="p-4 space-y-4">
      <div className="flex justify-between items-center"><h1 className="text-2xl font-bold">Vars</h1><div className="flex gap-2"><Badge>{total} total</Badge><Button onClick={startCreate}>Add Vars</Button></div></div>

      <div className="flex gap-2">
        <Input value={q} onChange={(e) => setQ(e.target.value)} placeholder="Search..." className="flex-1" />
        <Button variant="outline" onClick={() => { setPage(1); load(); }}>Search</Button>
      </div>

      <Card><CardHeader><CardTitle>Vars</CardTitle></CardHeader><CardContent>
          <div className="hidden md:block"><Table><TableHeader><TableRow><TableHead>Var Category</TableHead><TableHead>Var Name</TableHead><TableHead>Var Value</TableHead><TableHead>Enabled</TableHead><TableHead>Actions</TableHead></TableRow></TableHeader><TableBody>
                {data.map((d: any, i: number) => {
                  const uid = d.var_uuid || d.id || i;
                  return (
                    <TableRow key={String(uid)}>
                      <TableCell className="text-xs">{d.var_category ?? "-"}</TableCell>
                      <TableCell className="font-mono text-xs">{d.var_name ?? d["name"] ?? "-"}</TableCell>
                      <TableCell className="text-xs">{d.var_value ?? "-"}</TableCell>
                      <TableCell>{(d.var_enabled ?? true) ? <Badge>enabled</Badge> : <Badge className="bg-muted text-muted-foreground">disabled</Badge>}</TableCell>
                      <TableCell><div className="flex gap-1"><Button variant="ghost" aria-label="Edit" title="Edit" className="h-9 w-9 p-0" onClick={() => startEdit(d)}><Pencil className="h-4 w-4" /></Button><Button variant="ghost" aria-label="Delete" title="Delete" className="h-9 w-9 p-0 text-destructive hover:text-destructive hover:bg-destructive/10" onClick={() => del(d)}><Trash2 className="h-4 w-4" /></Button></div></TableCell>
                    </TableRow>
                  );
                })}
                {data.length===0 && <TableRow><TableCell colSpan={5} className="text-center text-muted-foreground py-8">No vars — create via Add button</TableCell></TableRow>}
              </TableBody></Table></div>
          <div className="md:hidden grid gap-3">
            {data.map((d: any, i: number) => {
              const uid = d.var_uuid || d.id || i;
              return (
                <Card key={String(uid)} className="p-3 space-y-1">
                  <div className="flex justify-between items-start"><div className="font-mono font-bold">{d.var_category ?? d["category"] ?? String(uid).slice(0,8)}</div><Badge>{(d.var_enabled ?? true) ? "enabled" : "disabled"}</Badge></div>
                  <div className="font-mono text-xs">{d.var_category ?? d["category"]}</div>
                  <div className="font-mono text-xs">{d.var_name ?? d["name"]}</div>
                  <div className="text-xs text-muted-foreground truncate">{d.var_name ?? ""}</div>
                  <div className="flex gap-2 mt-2"><Button variant="ghost" aria-label="Edit" title="Edit" className="flex-1 h-9" onClick={() => startEdit(d)}><Pencil className="h-4 w-4" /></Button><Button variant="ghost" aria-label="Delete" title="Delete" className="flex-1 h-9 text-destructive hover:text-destructive hover:bg-destructive/10" onClick={() => del(d)}><Trash2 className="h-4 w-4" /></Button></div>
                </Card>
              );
            })}
            {data.length===0 && <div className="text-center text-muted-foreground py-4">No vars</div>}
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
      <Dialog open={open} onOpenChange={(v) => { setOpen(v); if (!v) setEditing(null); }}><DialogHeader><DialogTitle>{editing ? "Edit Vars" : "Add Vars"}</DialogTitle></DialogHeader>
        <div className="space-y-3">
          <div><Label>Var Category</Label><Input value={form.var_category} onChange={(e) => setForm({ ...form, var_category: e.target.value })} placeholder="var_category" /></div>
          <div><Label>Var Name</Label><Input value={form.var_name} onChange={(e) => setForm({ ...form, var_name: e.target.value })} placeholder="e.g., test" /></div>
          <div><Label>Var Value</Label><Input value={form.var_value} onChange={(e) => setForm({ ...form, var_value: e.target.value })} placeholder="var_value" /></div>
          <div className="flex items-center gap-2"><input type="checkbox" id="var_enabled" checked={form.var_enabled as boolean} onChange={(e) => setForm({ ...form, var_enabled: e.target.checked })} className="h-4 w-4" /><Label htmlFor="var_enabled" className="cursor-pointer">Var Enabled</Label></div>
          <div><Label>Var Description</Label><Input value={form.var_description} onChange={(e) => setForm({ ...form, var_description: e.target.value })} placeholder="Description" /></div>
          <Button onClick={createOrUpdate} className="w-full">{editing ? "Save" : "Create"}</Button>
        </div></Dialog>
    </div>
  );
}
