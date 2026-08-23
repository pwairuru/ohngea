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

export function ConferenceProfiles() {
  const [data, setData] = React.useState<any[]>([]);
  const [total, setTotal] = React.useState(0);
  const [q, setQ] = React.useState("");
  const [page, setPage] = React.useState(1);
  const [open, setOpen] = React.useState(false);
  const [editing, setEditing] = React.useState<any | null>(null);
  const [form, setForm] = React.useState({
    profile_name: "test",
    profile_enabled: true,
    profile_description: "",
    params: ""
  });

  const load = React.useCallback(() => {
    const params = new URLSearchParams({ page: String(page), per_page: "20" });
    if (q) params.set("q", q);
    apiFetch(`/conference_profiles?${params.toString()}`)
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
    profile_name: "test",
    profile_enabled: true,
    profile_description: "",
    params: ""
    });
    setOpen(true);
  };

  const startEdit = (row: any) => {
    setEditing(row);
    setForm({
      profile_name: row.profile_name ?? row["name"] ?? "",
      profile_enabled: row.profile_enabled ?? row["enabled"] ?? true,
      profile_description: row.profile_description ?? row["description"] ?? "",
      params: row.params ?? row["params"] ?? "",
    });
    setOpen(true);
  };

  const createOrUpdate = async () => {
    try {
      const payload: any = {
        profile_name: form.profile_name || null,
        profile_enabled: form.profile_enabled,
        profile_description: form.profile_description || null,
        params: form.params || null,
        name: form.profile_name || null,
        enabled: form.profile_enabled || null,
        description: form.profile_description || null,
      };
      if (editing) {
        const uid = editing.conference_profile_uuid || editing.id || editing.uuid;
        await apiFetch(`/conference_profiles/${uid}`, { method: "PATCH", body: JSON.stringify(payload) });
      } else {
        await apiFetch("/conference_profiles", { method: "POST", body: JSON.stringify(payload) });
      }
      setOpen(false);
      setEditing(null);
      load();
    } catch (e: any) {
      alert(e.message);
    }
  };

  const del = async (row: any) => {
    const uid = row.conference_profile_uuid || row.id || row.uuid;
    const label = row.profile_name || uid;
    if (!confirm(`Delete ${label}?`)) return;
    try {
      await apiFetch(`/conference_profiles/${uid}`, { method: "DELETE" });
      load();
    } catch (e: any) {
      alert(e.message);
    }
  };

  return (
    <div className="p-4 space-y-4">
      <div className="flex justify-between items-center"><h1 className="text-2xl font-bold">Conference Profiles</h1><div className="flex gap-2"><Badge>{total} total</Badge><Button onClick={startCreate}>Add Conference Profiles</Button></div></div>

      <div className="flex gap-2">
        <Input value={q} onChange={(e) => setQ(e.target.value)} placeholder="Search..." className="flex-1" />
        <Button variant="outline" onClick={() => { setPage(1); load(); }}>Search</Button>
      </div>

      <Card><CardHeader><CardTitle>Conference Profiles</CardTitle></CardHeader><CardContent>
          <div className="hidden md:block"><Table><TableHeader><TableRow><TableHead>Profile Name</TableHead><TableHead>Profile Description</TableHead><TableHead>Params</TableHead><TableHead>Enabled</TableHead><TableHead>Actions</TableHead></TableRow></TableHeader><TableBody>
                {data.map((d: any, i: number) => {
                  const uid = d.conference_profile_uuid || d.id || i;
                  return (
                    <TableRow key={String(uid)}>
                      <TableCell className="font-mono text-xs">{d.profile_name ?? d["name"] ?? "-"}</TableCell>
                      <TableCell className="text-xs text-muted-foreground max-w-[200px] truncate" title={d.profile_description || ""}>{d.profile_description ?? "-"}</TableCell>
                      <TableCell className="text-xs">{d.params ?? "-"}</TableCell>
                      <TableCell>{(d.profile_enabled ?? true) ? <Badge>enabled</Badge> : <Badge className="bg-muted text-muted-foreground">disabled</Badge>}</TableCell>
                      <TableCell><div className="flex gap-1"><Button variant="ghost" aria-label="Edit" title="Edit" className="h-9 w-9 p-0" onClick={() => startEdit(d)}><Pencil className="h-4 w-4" /></Button><Button variant="ghost" aria-label="Delete" title="Delete" className="h-9 w-9 p-0 text-destructive hover:text-destructive hover:bg-destructive/10" onClick={() => del(d)}><Trash2 className="h-4 w-4" /></Button></div></TableCell>
                    </TableRow>
                  );
                })}
                {data.length===0 && <TableRow><TableCell colSpan={5} className="text-center text-muted-foreground py-8">No conference profiles — create via Add button</TableCell></TableRow>}
              </TableBody></Table></div>
          <div className="md:hidden grid gap-3">
            {data.map((d: any, i: number) => {
              const uid = d.conference_profile_uuid || d.id || i;
              return (
                <Card key={String(uid)} className="p-3 space-y-1">
                  <div className="flex justify-between items-start"><div className="font-mono font-bold">{d.profile_name ?? d["name"] ?? String(uid).slice(0,8)}</div><Badge>{(d.profile_enabled ?? true) ? "enabled" : "disabled"}</Badge></div>
                  <div className="font-mono text-xs">{d.profile_name ?? d["name"]}</div>
                  <div className="font-mono text-xs">{d.profile_description ?? d["description"]}</div>
                  <div className="text-xs text-muted-foreground truncate">{d.profile_description ?? ""}</div>
                  <div className="flex gap-2 mt-2"><Button variant="ghost" aria-label="Edit" title="Edit" className="flex-1 h-9" onClick={() => startEdit(d)}><Pencil className="h-4 w-4" /></Button><Button variant="ghost" aria-label="Delete" title="Delete" className="flex-1 h-9 text-destructive hover:text-destructive hover:bg-destructive/10" onClick={() => del(d)}><Trash2 className="h-4 w-4" /></Button></div>
                </Card>
              );
            })}
            {data.length===0 && <div className="text-center text-muted-foreground py-4">No conference profiles</div>}
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
      <Dialog open={open} onOpenChange={(v) => { setOpen(v); if (!v) setEditing(null); }}><DialogHeader><DialogTitle>{editing ? "Edit Conference Profiles" : "Add Conference Profiles"}</DialogTitle></DialogHeader>
        <div className="space-y-3">
          <div><Label>Profile Name *</Label><Input value={form.profile_name} onChange={(e) => setForm({ ...form, profile_name: e.target.value })} placeholder="e.g., test" /></div>
          <div className="flex items-center gap-2"><input type="checkbox" id="profile_enabled" checked={form.profile_enabled as boolean} onChange={(e) => setForm({ ...form, profile_enabled: e.target.checked })} className="h-4 w-4" /><Label htmlFor="profile_enabled" className="cursor-pointer">Profile Enabled</Label></div>
          <div><Label>Profile Description</Label><Input value={form.profile_description} onChange={(e) => setForm({ ...form, profile_description: e.target.value })} placeholder="Description" /></div>
          <div><Label>Params</Label><Input value={form.params} onChange={(e) => setForm({ ...form, params: e.target.value })} placeholder="params" /></div>
          <Button onClick={createOrUpdate} className="w-full">{editing ? "Save" : "Create"}</Button>
        </div></Dialog>
    </div>
  );
}
