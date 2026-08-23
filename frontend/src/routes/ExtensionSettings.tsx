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

export function ExtensionSettings() {
  const [data, setData] = React.useState<any[]>([]);
  const [total, setTotal] = React.useState(0);
  const [q, setQ] = React.useState("");
  const [page, setPage] = React.useState(1);
  const [open, setOpen] = React.useState(false);
  const [editing, setEditing] = React.useState<any | null>(null);
  const [form, setForm] = React.useState({
    extension_uuid: "1001",
    extension_setting_type: "1001",
    extension_setting_name: "test",
    extension_setting_value: "1001",
    extension_setting_enabled: true,
    extension_setting_description: "1001"
  });

  const load = React.useCallback(() => {
    const params = new URLSearchParams({ page: String(page), per_page: "20" });
    if (q) params.set("q", q);
    apiFetch(`/extension_settings?${params.toString()}`)
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
    extension_uuid: "1001",
    extension_setting_type: "1001",
    extension_setting_name: "test",
    extension_setting_value: "1001",
    extension_setting_enabled: true,
    extension_setting_description: "1001"
    });
    setOpen(true);
  };

  const startEdit = (row: any) => {
    setEditing(row);
    setForm({
      extension_uuid: row.extension_uuid ?? row["uuid"] ?? "",
      extension_setting_type: row.extension_setting_type ?? row["type"] ?? "",
      extension_setting_name: row.extension_setting_name ?? row["name"] ?? "",
      extension_setting_value: row.extension_setting_value ?? row["value"] ?? "",
      extension_setting_enabled: row.extension_setting_enabled ?? row["enabled"] ?? true,
      extension_setting_description: row.extension_setting_description ?? row["description"] ?? "",
    });
    setOpen(true);
  };

  const createOrUpdate = async () => {
    try {
      const payload: any = {
        extension_uuid: form.extension_uuid || null,
        extension_setting_type: form.extension_setting_type || null,
        extension_setting_name: form.extension_setting_name || null,
        extension_setting_value: form.extension_setting_value || null,
        extension_setting_enabled: form.extension_setting_enabled,
        extension_setting_description: form.extension_setting_description || null,
        name: form.extension_setting_name || null,
        value: form.extension_setting_value || null,
        enabled: form.extension_setting_enabled || null,
        description: form.extension_setting_description || null,
      };
      if (editing) {
        const uid = editing.extension_setting_uuid || editing.id || editing.uuid;
        await apiFetch(`/extension_settings/${uid}`, { method: "PATCH", body: JSON.stringify(payload) });
      } else {
        await apiFetch("/extension_settings", { method: "POST", body: JSON.stringify(payload) });
      }
      setOpen(false);
      setEditing(null);
      load();
    } catch (e: any) {
      alert(e.message);
    }
  };

  const del = async (row: any) => {
    const uid = row.extension_setting_uuid || row.id || row.uuid;
    const label = row.extension_uuid || uid;
    if (!confirm(`Delete ${label}?`)) return;
    try {
      await apiFetch(`/extension_settings/${uid}`, { method: "DELETE" });
      load();
    } catch (e: any) {
      alert(e.message);
    }
  };

  return (
    <div className="p-4 space-y-4">
      <div className="flex justify-between items-center"><h1 className="text-2xl font-bold">Extension Settings</h1><div className="flex gap-2"><Badge>{total} total</Badge><Button onClick={startCreate}>Add Extension Settings</Button></div></div>

      <div className="flex gap-2">
        <Input value={q} onChange={(e) => setQ(e.target.value)} placeholder="Search..." className="flex-1" />
        <Button variant="outline" onClick={() => { setPage(1); load(); }}>Search</Button>
      </div>

      <Card><CardHeader><CardTitle>Extension Settings</CardTitle></CardHeader><CardContent>
          <div className="hidden md:block"><Table><TableHeader><TableRow><TableHead>Extension Uuid</TableHead><TableHead>Extension Setting Type</TableHead><TableHead>Extension Setting Name</TableHead><TableHead>Enabled</TableHead><TableHead>Actions</TableHead></TableRow></TableHeader><TableBody>
                {data.map((d: any, i: number) => {
                  const uid = d.extension_setting_uuid || d.id || i;
                  return (
                    <TableRow key={String(uid)}>
                      <TableCell className="font-mono text-xs">{d.extension_uuid ?? d["uuid"] ?? "-"}</TableCell>
                      <TableCell className="font-mono text-xs">{d.extension_setting_type ?? d["type"] ?? "-"}</TableCell>
                      <TableCell className="font-mono text-xs">{d.extension_setting_name ?? d["name"] ?? "-"}</TableCell>
                      <TableCell>{(d.extension_setting_enabled ?? true) ? <Badge>enabled</Badge> : <Badge className="bg-muted text-muted-foreground">disabled</Badge>}</TableCell>
                      <TableCell><div className="flex gap-1"><Button variant="ghost" aria-label="Edit" title="Edit" className="h-9 w-9 p-0" onClick={() => startEdit(d)}><Pencil className="h-4 w-4" /></Button><Button variant="ghost" aria-label="Delete" title="Delete" className="h-9 w-9 p-0 text-destructive hover:text-destructive hover:bg-destructive/10" onClick={() => del(d)}><Trash2 className="h-4 w-4" /></Button></div></TableCell>
                    </TableRow>
                  );
                })}
                {data.length===0 && <TableRow><TableCell colSpan={5} className="text-center text-muted-foreground py-8">No extension settings — create via Add button</TableCell></TableRow>}
              </TableBody></Table></div>
          <div className="md:hidden grid gap-3">
            {data.map((d: any, i: number) => {
              const uid = d.extension_setting_uuid || d.id || i;
              return (
                <Card key={String(uid)} className="p-3 space-y-1">
                  <div className="flex justify-between items-start"><div className="font-mono font-bold">{d.extension_uuid ?? d["uuid"] ?? String(uid).slice(0,8)}</div><Badge>{(d.extension_setting_enabled ?? true) ? "enabled" : "disabled"}</Badge></div>
                  <div className="font-mono text-xs">{d.extension_uuid ?? d["uuid"]}</div>
                  <div className="font-mono text-xs">{d.extension_setting_type ?? d["type"]}</div>
                  <div className="text-xs text-muted-foreground truncate">{d.extension_setting_type ?? ""}</div>
                  <div className="flex gap-2 mt-2"><Button variant="ghost" aria-label="Edit" title="Edit" className="flex-1 h-9" onClick={() => startEdit(d)}><Pencil className="h-4 w-4" /></Button><Button variant="ghost" aria-label="Delete" title="Delete" className="flex-1 h-9 text-destructive hover:text-destructive hover:bg-destructive/10" onClick={() => del(d)}><Trash2 className="h-4 w-4" /></Button></div>
                </Card>
              );
            })}
            {data.length===0 && <div className="text-center text-muted-foreground py-4">No extension settings</div>}
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
      <Dialog open={open} onOpenChange={(v) => { setOpen(v); if (!v) setEditing(null); }}><DialogHeader><DialogTitle>{editing ? "Edit Extension Settings" : "Add Extension Settings"}</DialogTitle></DialogHeader>
        <div className="space-y-3">
          <div><Label>Extension Uuid</Label><Input value={form.extension_uuid} onChange={(e) => setForm({ ...form, extension_uuid: e.target.value })} placeholder="1001" /></div>
          <div><Label>Extension Setting Type</Label><Input value={form.extension_setting_type} onChange={(e) => setForm({ ...form, extension_setting_type: e.target.value })} placeholder="1001" /></div>
          <div><Label>Extension Setting Name</Label><Input value={form.extension_setting_name} onChange={(e) => setForm({ ...form, extension_setting_name: e.target.value })} placeholder="e.g., test" /></div>
          <div><Label>Extension Setting Value</Label><Input value={form.extension_setting_value} onChange={(e) => setForm({ ...form, extension_setting_value: e.target.value })} placeholder="1001" /></div>
          <div className="flex items-center gap-2"><input type="checkbox" id="extension_setting_enabled" checked={form.extension_setting_enabled as boolean} onChange={(e) => setForm({ ...form, extension_setting_enabled: e.target.checked })} className="h-4 w-4" /><Label htmlFor="extension_setting_enabled" className="cursor-pointer">Extension Setting Enabled</Label></div>
          <div><Label>Extension Setting Description</Label><Input value={form.extension_setting_description} onChange={(e) => setForm({ ...form, extension_setting_description: e.target.value })} placeholder="1001" /></div>
          <Button onClick={createOrUpdate} className="w-full">{editing ? "Save" : "Create"}</Button>
        </div></Dialog>
    </div>
  );
}
