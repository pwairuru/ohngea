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

export function NumberTranslations() {
  const [data, setData] = React.useState<any[]>([]);
  const [total, setTotal] = React.useState(0);
  const [q, setQ] = React.useState("");
  const [page, setPage] = React.useState(1);
  const [open, setOpen] = React.useState(false);
  const [editing, setEditing] = React.useState<any | null>(null);
  const [form, setForm] = React.useState({
    number_translation_name: "test",
    number_translation_regex: "1001",
    number_translation_replace: "1001",
    number_translation_enabled: true,
    number_translation_description: "1001"
  });

  const load = React.useCallback(() => {
    const params = new URLSearchParams({ page: String(page), per_page: "20" });
    if (q) params.set("q", q);
    apiFetch(`/number_translations?${params.toString()}`)
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
    number_translation_name: "test",
    number_translation_regex: "1001",
    number_translation_replace: "1001",
    number_translation_enabled: true,
    number_translation_description: "1001"
    });
    setOpen(true);
  };

  const startEdit = (row: any) => {
    setEditing(row);
    setForm({
      number_translation_name: row.number_translation_name ?? row["name"] ?? "",
      number_translation_regex: row.number_translation_regex ?? row["regex"] ?? "",
      number_translation_replace: row.number_translation_replace ?? row["replace"] ?? "",
      number_translation_enabled: row.number_translation_enabled ?? row["enabled"] ?? true,
      number_translation_description: row.number_translation_description ?? row["description"] ?? "",
    });
    setOpen(true);
  };

  const createOrUpdate = async () => {
    try {
      const payload: any = {
        number_translation_name: form.number_translation_name || null,
        number_translation_regex: form.number_translation_regex || null,
        number_translation_replace: form.number_translation_replace || null,
        number_translation_enabled: form.number_translation_enabled,
        number_translation_description: form.number_translation_description || null,
        name: form.number_translation_name || null,
        enabled: form.number_translation_enabled || null,
        description: form.number_translation_description || null,
      };
      if (editing) {
        const uid = editing.number_translation_uuid || editing.id || editing.uuid;
        await apiFetch(`/number_translations/${uid}`, { method: "PATCH", body: JSON.stringify(payload) });
      } else {
        await apiFetch("/number_translations", { method: "POST", body: JSON.stringify(payload) });
      }
      setOpen(false);
      setEditing(null);
      load();
    } catch (e: any) {
      alert(e.message);
    }
  };

  const del = async (row: any) => {
    const uid = row.number_translation_uuid || row.id || row.uuid;
    const label = row.number_translation_name || uid;
    if (!confirm(`Delete ${label}?`)) return;
    try {
      await apiFetch(`/number_translations/${uid}`, { method: "DELETE" });
      load();
    } catch (e: any) {
      alert(e.message);
    }
  };

  return (
    <div className="p-4 space-y-4">
      <div className="flex justify-between items-center"><h1 className="text-2xl font-bold">Number Translations</h1><div className="flex gap-2"><Badge>{total} total</Badge><Button onClick={startCreate}>Add Number Translations</Button></div></div>

      <div className="flex gap-2">
        <Input value={q} onChange={(e) => setQ(e.target.value)} placeholder="Search..." className="flex-1" />
        <Button variant="outline" onClick={() => { setPage(1); load(); }}>Search</Button>
      </div>

      <Card><CardHeader><CardTitle>Number Translations</CardTitle></CardHeader><CardContent>
          <div className="hidden md:block"><Table><TableHeader><TableRow><TableHead>Number Translation Name</TableHead><TableHead>Number Translation Regex</TableHead><TableHead>Number Translation Replace</TableHead><TableHead>Enabled</TableHead><TableHead>Actions</TableHead></TableRow></TableHeader><TableBody>
                {data.map((d: any, i: number) => {
                  const uid = d.number_translation_uuid || d.id || i;
                  return (
                    <TableRow key={String(uid)}>
                      <TableCell className="font-mono text-xs">{d.number_translation_name ?? d["name"] ?? "-"}</TableCell>
                      <TableCell className="font-mono text-xs">{d.number_translation_regex ?? d["regex"] ?? "-"}</TableCell>
                      <TableCell className="font-mono text-xs">{d.number_translation_replace ?? d["replace"] ?? "-"}</TableCell>
                      <TableCell>{(d.number_translation_enabled ?? true) ? <Badge>enabled</Badge> : <Badge className="bg-muted text-muted-foreground">disabled</Badge>}</TableCell>
                      <TableCell><div className="flex gap-1"><Button variant="ghost" aria-label="Edit" title="Edit" className="h-9 w-9 p-0" onClick={() => startEdit(d)}><Pencil className="h-4 w-4" /></Button><Button variant="ghost" aria-label="Delete" title="Delete" className="h-9 w-9 p-0 text-destructive hover:text-destructive hover:bg-destructive/10" onClick={() => del(d)}><Trash2 className="h-4 w-4" /></Button></div></TableCell>
                    </TableRow>
                  );
                })}
                {data.length===0 && <TableRow><TableCell colSpan={5} className="text-center text-muted-foreground py-8">No number translations — create via Add button</TableCell></TableRow>}
              </TableBody></Table></div>
          <div className="md:hidden grid gap-3">
            {data.map((d: any, i: number) => {
              const uid = d.number_translation_uuid || d.id || i;
              return (
                <Card key={String(uid)} className="p-3 space-y-1">
                  <div className="flex justify-between items-start"><div className="font-mono font-bold">{d.number_translation_name ?? d["name"] ?? String(uid).slice(0,8)}</div><Badge>{(d.number_translation_enabled ?? true) ? "enabled" : "disabled"}</Badge></div>
                  <div className="font-mono text-xs">{d.number_translation_name ?? d["name"]}</div>
                  <div className="font-mono text-xs">{d.number_translation_regex ?? d["regex"]}</div>
                  <div className="text-xs text-muted-foreground truncate">{d.number_translation_regex ?? ""}</div>
                  <div className="flex gap-2 mt-2"><Button variant="ghost" aria-label="Edit" title="Edit" className="flex-1 h-9" onClick={() => startEdit(d)}><Pencil className="h-4 w-4" /></Button><Button variant="ghost" aria-label="Delete" title="Delete" className="flex-1 h-9 text-destructive hover:text-destructive hover:bg-destructive/10" onClick={() => del(d)}><Trash2 className="h-4 w-4" /></Button></div>
                </Card>
              );
            })}
            {data.length===0 && <div className="text-center text-muted-foreground py-4">No number translations</div>}
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
      <Dialog open={open} onOpenChange={(v) => { setOpen(v); if (!v) setEditing(null); }}><DialogHeader><DialogTitle>{editing ? "Edit Number Translations" : "Add Number Translations"}</DialogTitle></DialogHeader>
        <div className="space-y-3">
          <div><Label>Number Translation Name *</Label><Input value={form.number_translation_name} onChange={(e) => setForm({ ...form, number_translation_name: e.target.value })} placeholder="e.g., test" /></div>
          <div><Label>Number Translation Regex</Label><Input value={form.number_translation_regex} onChange={(e) => setForm({ ...form, number_translation_regex: e.target.value })} placeholder="1001" /></div>
          <div><Label>Number Translation Replace</Label><Input value={form.number_translation_replace} onChange={(e) => setForm({ ...form, number_translation_replace: e.target.value })} placeholder="1001" /></div>
          <div className="flex items-center gap-2"><input type="checkbox" id="number_translation_enabled" checked={form.number_translation_enabled as boolean} onChange={(e) => setForm({ ...form, number_translation_enabled: e.target.checked })} className="h-4 w-4" /><Label htmlFor="number_translation_enabled" className="cursor-pointer">Number Translation Enabled</Label></div>
          <div><Label>Number Translation Description</Label><Input value={form.number_translation_description} onChange={(e) => setForm({ ...form, number_translation_description: e.target.value })} placeholder="1001" /></div>
          <Button onClick={createOrUpdate} className="w-full">{editing ? "Save" : "Create"}</Button>
        </div></Dialog>
    </div>
  );
}
