import * as React from "react";
import { apiFetch } from "../lib/api";
import { Card, CardContent, CardHeader, CardTitle } from "../components/ui/card";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "../components/ui/table";
import { Badge } from "../components/ui/badge";
import { Button } from "../components/ui/button";
import { Input } from "../components/ui/input";
import { Label } from "../components/ui/label";
import { Dialog, DialogHeader, DialogTitle } from "../components/ui/dialog";
import {Pencil, Trash2, ChevronsLeft, ChevronLeft, ChevronRight, ChevronsRight} from "lucide-react";

export function AccessControls() {
  const [data, setData] = React.useState<any[]>([]);
  const [total, setTotal] = React.useState(0);
  const [q, setQ] = React.useState("");
  const [page, setPage] = React.useState(1);
  const [open, setOpen] = React.useState(false);
  const [editing, setEditing] = React.useState<any | null>(null);
  const [form, setForm] = React.useState({ access_control_name: "lan", access_control_default: "deny", access_control_description: "" });

  const load = React.useCallback(() => {
    const params = new URLSearchParams({ page: String(page), per_page: "20" });
    if (q) params.set("q", q);
    apiFetch(`/access_controls?${params.toString()}`)
      .then((r: any) => {
        setData(r.data || []);
        setTotal(r.total ?? r.data?.length ?? 0);
      })
      .catch(() => {});
  }, [page, q]);

  React.useEffect(() => {
    load();
  }, [load]);

  const startCreate = () => {
    setEditing(null);
    setForm({ access_control_name: "lan", access_control_default: "deny", access_control_description: "" });
    setOpen(true);
  };

  const startEdit = (row: any) => {
    setEditing(row);
    setForm({
      access_control_name: row.access_control_name || row.name || "",
      access_control_default: row.access_control_default || row.default || "deny",
      access_control_description: row.access_control_description || row.description || "",
    });
    setOpen(true);
  };

  const createOrUpdate = async () => {
    try {
      const payload: any = {
        access_control_name: form.access_control_name,
        access_control_default: form.access_control_default,
        access_control_description: form.access_control_description || null,
        // aliases for robustness
        name: form.access_control_name,
        default: form.access_control_default,
        description: form.access_control_description || null,
      };
      if (editing) {
        const uid = editing.access_control_uuid || editing.id;
        await apiFetch(`/access_controls/${uid}`, { method: "PATCH", body: JSON.stringify(payload) });
      } else {
        await apiFetch("/access_controls", { method: "POST", body: JSON.stringify(payload) });
      }
      setOpen(false);
      setEditing(null);
      load();
    } catch (e: any) {
      alert(e.message);
    }
  };

  const del = async (row: any) => {
    const uid = row.access_control_uuid || row.id;
    const name = row.access_control_name || row.name || uid;
    if (!confirm(`Delete access control ${name}?`)) return;
    try {
      await apiFetch(`/access_controls/${uid}`, { method: "DELETE" });
      load();
    } catch (e: any) {
      alert(e.message);
    }
  };

  return (
    <div className="p-4 space-y-4">
      <div className="flex justify-between items-center">
        <h1 className="text-2xl font-bold">Access Controls</h1>
        <div className="flex gap-2">
          <Badge>{total} total</Badge>
          <Button onClick={startCreate}>Add Access Control</Button>
        </div>
      </div>

      <div className="flex gap-2">
        <Input value={q} onChange={(e) => setQ(e.target.value)} placeholder="Search name, description..." className="flex-1" />
        <Button variant="outline" onClick={() => { setPage(1); load(); }}>Search</Button>
      </div>

      <Card>
        <CardHeader><CardTitle>Access Controls — CIDR allow/deny lists</CardTitle></CardHeader>
        <CardContent>
          <div className="hidden md:block">
            <Table>
              <TableHeader><TableRow><TableHead>Name</TableHead><TableHead>Default</TableHead><TableHead>Description</TableHead><TableHead>Nodes</TableHead><TableHead>Actions</TableHead></TableRow></TableHeader>
              <TableBody>
                {data.map((d: any) => {
                  const uid = d.access_control_uuid || d.id || d.access_control_name;
                  const nodes = d.nodes || [];
                  return (
                    <TableRow key={String(uid)}>
                      <TableCell className="font-mono font-bold">{d.access_control_name ?? d.name}</TableCell>
                      <TableCell><Badge className={d.access_control_default === "allow" ? "bg-green-600" : "bg-muted text-muted-foreground"}>{d.access_control_default ?? d.default ?? "deny"}</Badge></TableCell>
                      <TableCell className="text-xs text-muted-foreground max-w-[260px] truncate" title={d.access_control_description || d.description}>{d.access_control_description || d.description || "-"}</TableCell>
                      <TableCell className="text-xs">{Array.isArray(nodes) ? nodes.length : 0}</TableCell>
                      <TableCell>
                        <div className="flex gap-1">
                          <Button variant="ghost" aria-label={`Edit ${d.access_control_name ?? d.name}`} title="Edit" className="h-9 w-9 p-0" onClick={() => startEdit(d)}><Pencil className="h-4 w-4" /></Button>
                          <Button variant="ghost" aria-label={`Delete ${d.access_control_name ?? d.name}`} title="Delete" className="h-9 w-9 p-0 text-destructive hover:text-destructive hover:bg-destructive/10" onClick={() => del(d)}><Trash2 className="h-4 w-4" /></Button>
                        </div>
                      </TableCell>
                    </TableRow>
                  );
                })}
                {data.length === 0 && <TableRow><TableCell colSpan={5} className="text-center text-muted-foreground py-8">No access controls — create via Add button</TableCell></TableRow>}
              </TableBody>
            </Table>
          </div>
          <div className="md:hidden grid gap-3">
            {data.map((d: any) => {
              const uid = d.access_control_uuid || d.id;
              return (
                <Card key={String(uid)} className="p-3 space-y-1">
                  <div className="flex justify-between items-center"><div className="font-mono font-bold">{d.access_control_name}</div><Badge>{d.access_control_default}</Badge></div>
                  <div className="text-xs text-muted-foreground">{d.access_control_description || "no description"}</div>
                  <div className="text-xs">nodes: {(d.nodes || []).length}</div>
                  <div className="flex gap-2 mt-2"><Button variant="ghost" aria-label={`Edit ${d.access_control_name}`} title="Edit" className="flex-1 h-9" onClick={() => startEdit(d)}><Pencil className="h-4 w-4" /></Button><Button variant="ghost" aria-label={`Delete ${d.access_control_name}`} title="Delete" className="flex-1 h-9 text-destructive hover:text-destructive hover:bg-destructive/10" onClick={() => del(d)}><Trash2 className="h-4 w-4" /></Button></div>
                </Card>
              );
            })}
            {data.length === 0 && <div className="text-center text-muted-foreground py-4">No access controls</div>}
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
        </CardContent>
      </Card>

      <Dialog open={open} onOpenChange={(v) => { setOpen(v); if (!v) setEditing(null); }}>
        <DialogHeader><DialogTitle>{editing ? "Edit Access Control" : "Add Access Control"}</DialogTitle></DialogHeader>
        <div className="space-y-3">
          <div><Label>Name *</Label><Input value={form.access_control_name} onChange={(e) => setForm({ ...form, access_control_name: e.target.value })} placeholder="lan" /></div>
          <div>
            <Label>Default *</Label>
            <select value={form.access_control_default} onChange={(e) => setForm({ ...form, access_control_default: e.target.value })} className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm min-h-[44px]">
              <option value="deny">deny</option>
              <option value="allow">allow</option>
            </select>
            <div className="text-xs text-muted-foreground mt-1">Default policy for unmatched CIDR — usually deny</div>
          </div>
          <div><Label>Description</Label><Input value={form.access_control_description} onChange={(e) => setForm({ ...form, access_control_description: e.target.value })} placeholder="Local network ACL" /></div>
          <Button onClick={createOrUpdate} className="w-full">{editing ? "Save" : "Create"}</Button>
        </div>
      </Dialog>
    </div>
  );
}
