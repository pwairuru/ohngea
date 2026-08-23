import * as React from "react";
import { EmptyState } from "../../components/ui/empty-state";
import { toast, toastError } from "../../lib/toast";
import { apiFetch } from "../../lib/api";
import { Card, CardContent, CardHeader, CardTitle } from "../../components/ui/card";
import { Button } from "../../components/ui/button";
import { Input } from "../../components/ui/input";
import { Label } from "../../components/ui/label";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "../../components/ui/table";
import { Badge } from "../../components/ui/badge";

export default function FaxPlugin() {
  const [items, setItems] = React.useState<any[]>([]);
  const [total, setTotal] = React.useState(0);
  const [q, setQ] = React.useState("");
  const [form, setForm] = React.useState({ fax_extension: "", fax_name: "", fax_email: "", enabled: true });
  const load = () =>
    apiFetch(`/fax?page=1&per_page=20&q=${encodeURIComponent(q)}`)
      .then((r: any) => {
        setItems(r.data || []);
        setTotal(r.total || 0);
      })
      .catch(() => {});
  React.useEffect(() => { load(); }, []);
  const create = async () => {
    const me: any = await apiFetch("/auth/me");
    const domain_id = me.domain_id;
    if (!domain_id) {
      toastError(new Error("No domain found — create domain first"));
      return;
    }
    await apiFetch("/fax", {
      method: "POST",
      body: JSON.stringify({ ...form, domain_id }),
    });
    setForm({ fax_extension: "", fax_name: "", fax_email: "", enabled: true });
    load();
  };
  return (
    <div className="p-4 space-y-4">
      <h1 className="text-2xl font-bold">Fax <Badge>core</Badge> <Badge variant="outline">MIT</Badge></h1>
      <Card><CardHeader><CardTitle>Fax Extensions — per-tenant, RLS + hash partitioned</CardTitle></CardHeader><CardContent className="text-sm text-muted-foreground">Manage fax extensions per domain. `POST /api/v1/fax` creates `fax_extension` 2-7 digits, `fax_name`, `fax_email` for delivery. Worker `app/workers/celery_app.py` handles `mod_spandsp` via `fax_send`.</CardContent></Card>
      <div className="flex gap-2">
        <Input value={q} onChange={(e) => setQ(e.target.value)} placeholder="Search fax name, extension..." className="flex-1" />
        <Button onClick={load}>Search</Button>
      </div>
      <Card><CardHeader><CardTitle>Add Fax Extension</CardTitle></CardHeader><CardContent className="space-y-3">
        <div><Label>Fax Extension *</Label><Input value={form.fax_extension} onChange={(e) => setForm({ ...form, fax_extension: e.target.value })} placeholder="300" /></div>
        <div><Label>Fax Name *</Label><Input value={form.fax_name} onChange={(e) => setForm({ ...form, fax_name: e.target.value })} placeholder="Fax 300" /></div>
        <div><Label>Email</Label><Input value={form.fax_email} onChange={(e) => setForm({ ...form, fax_email: e.target.value })} placeholder="fax@example.com" /></div>
        <Button onClick={create} className="w-full">Create</Button>
      </CardContent></Card>
      <Card><CardHeader><CardTitle>Fax Extensions — total {total}</CardTitle></CardHeader><CardContent>
        <div className="hidden md:block"><Table><TableHeader><TableRow><TableHead>Extension</TableHead><TableHead>Name</TableHead><TableHead>Email</TableHead><TableHead>Status</TableHead></TableRow></TableHeader><TableBody>{items.map((it: any) => <TableRow key={it.fax_uuid}><TableCell className="font-mono">{it.fax_extension}</TableCell><TableCell>{it.fax_name}</TableCell><TableCell>{it.fax_email || "-"}</TableCell><TableCell><Badge>{it.enabled ? "enabled" : "disabled"}</Badge></TableCell></TableRow>)}{items.length===0 && <TableRow><TableCell colSpan={4}><EmptyState title="No fax extensions" /></TableCell></TableRow>}</TableBody></Table></div>
        <div className="md:hidden grid gap-2">{items.map((it: any) => <Card key={it.fax_uuid} className="p-3"><div className="font-mono font-bold">{it.fax_extension}</div><div className="text-sm">{it.fax_name}</div><div className="text-xs text-muted-foreground">{it.fax_email}</div><Badge>{it.enabled ? "enabled" : "disabled"}</Badge></Card>)}{items.length===0 && <EmptyState title="No faxes" />}</div>
      </CardContent></Card>
    </div>
  );
}
