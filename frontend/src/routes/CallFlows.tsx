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

export function CallFlows() {
  const [data, setData] = React.useState<any[]>([]);
  const [total, setTotal] = React.useState(0);
  const [q, setQ] = React.useState("");
  const [page, setPage] = React.useState(1);
  const [open, setOpen] = React.useState(false);
  const [editing, setEditing] = React.useState<any | null>(null);
  const [form, setForm] = React.useState({
    call_flow_name: "test",
    call_flow_extension: "1001",
    call_flow_feature_code: "",
    feature_code: "",
    call_flow_context: "public",
    call_flow_enabled: true,
    call_flow_description: ""
  });

  const load = React.useCallback(() => {
    const params = new URLSearchParams({ page: String(page), per_page: "20" });
    if (q) params.set("q", q);
    apiFetch(`/call_flows?${params.toString()}`)
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
    call_flow_name: "test",
    call_flow_extension: "1001",
    call_flow_feature_code: "",
    feature_code: "",
    call_flow_context: "public",
    call_flow_enabled: true,
    call_flow_description: ""
    });
    setOpen(true);
  };

  const startEdit = (row: any) => {
    setEditing(row);
    setForm({
      call_flow_name: row.call_flow_name ?? row["name"] ?? "",
      call_flow_extension: row.call_flow_extension ?? row["extension"] ?? "",
      call_flow_feature_code: row.call_flow_feature_code ?? row["code"] ?? "",
      feature_code: row.feature_code ?? row["code"] ?? "",
      call_flow_context: row.call_flow_context ?? row["context"] ?? "",
      call_flow_enabled: row.call_flow_enabled ?? row["enabled"] ?? true,
      call_flow_description: row.call_flow_description ?? row["description"] ?? "",
    });
    setOpen(true);
  };

  const createOrUpdate = async () => {
    try {
      const payload: any = {
        call_flow_name: form.call_flow_name || null,
        call_flow_extension: form.call_flow_extension || null,
        call_flow_feature_code: form.call_flow_feature_code || null,
        feature_code: form.feature_code || null,
        call_flow_context: form.call_flow_context || null,
        call_flow_enabled: form.call_flow_enabled,
        call_flow_description: form.call_flow_description || null,
        name: form.call_flow_name || null,
        enabled: form.call_flow_enabled || null,
        description: form.call_flow_description || null,
      };
      if (editing) {
        const uid = editing.call_flow_uuid || editing.id || editing.uuid;
        await apiFetch(`/call_flows/${uid}`, { method: "PATCH", body: JSON.stringify(payload) });
      } else {
        await apiFetch("/call_flows", { method: "POST", body: JSON.stringify(payload) });
      }
      setOpen(false);
      setEditing(null);
      load();
    } catch (e: any) {
      alert(e.message);
    }
  };

  const del = async (row: any) => {
    const uid = row.call_flow_uuid || row.id || row.uuid;
    const label = row.call_flow_name || uid;
    if (!confirm(`Delete ${label}?`)) return;
    try {
      await apiFetch(`/call_flows/${uid}`, { method: "DELETE" });
      load();
    } catch (e: any) {
      alert(e.message);
    }
  };

  return (
    <div className="p-4 space-y-4">
      <div className="flex justify-between items-center"><h1 className="text-2xl font-bold">Call Flows</h1><div className="flex gap-2"><Badge>{total} total</Badge><Button onClick={startCreate}>Add Call Flows</Button></div></div>

      <div className="flex gap-2">
        <Input value={q} onChange={(e) => setQ(e.target.value)} placeholder="Search..." className="flex-1" />
        <Button variant="outline" onClick={() => { setPage(1); load(); }}>Search</Button>
      </div>

      <Card><CardHeader><CardTitle>Call Flows</CardTitle></CardHeader><CardContent>
          <div className="hidden md:block"><Table><TableHeader><TableRow><TableHead>Call Flow Name</TableHead><TableHead>Call Flow Extension</TableHead><TableHead>Call Flow Feature Code</TableHead><TableHead>Enabled</TableHead><TableHead>Actions</TableHead></TableRow></TableHeader><TableBody>
                {data.map((d: any, i: number) => {
                  const uid = d.call_flow_uuid || d.id || i;
                  return (
                    <TableRow key={String(uid)}>
                      <TableCell className="font-mono text-xs">{d.call_flow_name ?? d["name"] ?? "-"}</TableCell>
                      <TableCell className="font-mono text-xs">{d.call_flow_extension ?? d["extension"] ?? "-"}</TableCell>
                      <TableCell className="text-xs">{d.call_flow_feature_code ?? "-"}</TableCell>
                      <TableCell>{(d.call_flow_enabled ?? true) ? <Badge>enabled</Badge> : <Badge className="bg-muted text-muted-foreground">disabled</Badge>}</TableCell>
                      <TableCell><div className="flex gap-1"><Button variant="ghost" aria-label="Edit" title="Edit" className="h-9 w-9 p-0" onClick={() => startEdit(d)}><Pencil className="h-4 w-4" /></Button><Button variant="ghost" aria-label="Delete" title="Delete" className="h-9 w-9 p-0 text-destructive hover:text-destructive hover:bg-destructive/10" onClick={() => del(d)}><Trash2 className="h-4 w-4" /></Button></div></TableCell>
                    </TableRow>
                  );
                })}
                {data.length===0 && <TableRow><TableCell colSpan={5} className="text-center text-muted-foreground py-8">No call flows — create via Add button</TableCell></TableRow>}
              </TableBody></Table></div>
          <div className="md:hidden grid gap-3">
            {data.map((d: any, i: number) => {
              const uid = d.call_flow_uuid || d.id || i;
              return (
                <Card key={String(uid)} className="p-3 space-y-1">
                  <div className="flex justify-between items-start"><div className="font-mono font-bold">{d.call_flow_name ?? d["name"] ?? String(uid).slice(0,8)}</div><Badge>{(d.call_flow_enabled ?? true) ? "enabled" : "disabled"}</Badge></div>
                  <div className="font-mono text-xs">{d.call_flow_name ?? d["name"]}</div>
                  <div className="font-mono text-xs">{d.call_flow_extension ?? d["extension"]}</div>
                  <div className="text-xs text-muted-foreground truncate">{d.call_flow_extension ?? ""}</div>
                  <div className="flex gap-2 mt-2"><Button variant="ghost" aria-label="Edit" title="Edit" className="flex-1 h-9" onClick={() => startEdit(d)}><Pencil className="h-4 w-4" /></Button><Button variant="ghost" aria-label="Delete" title="Delete" className="flex-1 h-9 text-destructive hover:text-destructive hover:bg-destructive/10" onClick={() => del(d)}><Trash2 className="h-4 w-4" /></Button></div>
                </Card>
              );
            })}
            {data.length===0 && <div className="text-center text-muted-foreground py-4">No call flows</div>}
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
      <Dialog open={open} onOpenChange={(v) => { setOpen(v); if (!v) setEditing(null); }}><DialogHeader><DialogTitle>{editing ? "Edit Call Flows" : "Add Call Flows"}</DialogTitle></DialogHeader>
        <div className="space-y-3">
          <div><Label>Call Flow Name *</Label><Input value={form.call_flow_name} onChange={(e) => setForm({ ...form, call_flow_name: e.target.value })} placeholder="e.g., test" /></div>
          <div><Label>Call Flow Extension</Label><Input value={form.call_flow_extension} onChange={(e) => setForm({ ...form, call_flow_extension: e.target.value })} placeholder="1001" /></div>
          <div><Label>Call Flow Feature Code</Label><Input value={form.call_flow_feature_code} onChange={(e) => setForm({ ...form, call_flow_feature_code: e.target.value })} placeholder="call_flow_feature_code" /></div>
          <div><Label>Feature Code</Label><Input value={form.feature_code} onChange={(e) => setForm({ ...form, feature_code: e.target.value })} placeholder="feature_code" /></div>
          <div><Label>Call Flow Context</Label><Input value={form.call_flow_context} onChange={(e) => setForm({ ...form, call_flow_context: e.target.value })} placeholder="call_flow_context" /></div>
          <div className="flex items-center gap-2"><input type="checkbox" id="call_flow_enabled" checked={form.call_flow_enabled as boolean} onChange={(e) => setForm({ ...form, call_flow_enabled: e.target.checked })} className="h-4 w-4" /><Label htmlFor="call_flow_enabled" className="cursor-pointer">Call Flow Enabled</Label></div>
          <div><Label>Call Flow Description</Label><Input value={form.call_flow_description} onChange={(e) => setForm({ ...form, call_flow_description: e.target.value })} placeholder="Description" /></div>
          <Button onClick={createOrUpdate} className="w-full">{editing ? "Save" : "Create"}</Button>
        </div></Dialog>
    </div>
  );
}
