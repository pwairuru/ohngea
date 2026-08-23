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

export function Voicemails() {
  const [data, setData] = React.useState<any[]>([]);
  const [total, setTotal] = React.useState(0);
  const [q, setQ] = React.useState("");
  const [page, setPage] = React.useState(1);
  const [open, setOpen] = React.useState(false);
  const [editing, setEditing] = React.useState<any | null>(null);
  const [form, setForm] = React.useState({
    voicemail_id: "1001",
    voicemail_password: "1234",
    description: "",
    greeting_id: "",
    toll_allow: "",
    voicemail_mail_to: "",
    forward_destinations: "",
    enabled: true,
  });

  const load = React.useCallback(() => {
    const params = new URLSearchParams({ page: String(page), per_page: "20" });
    if (q) params.set("q", q);
    apiFetch(`/voicemails?${params.toString()}`)
      .then((r: any) => { setData(r.data || []); setTotal(r.total ?? r.data?.length ?? 0); })
      .catch(() => {});
  }, [page, q]);

  React.useEffect(() => { load(); }, [load]);

  const startCreate = () => {
    setEditing(null);
    setForm({ voicemail_id: "1001", voicemail_password: "1234", description: "", greeting_id: "", toll_allow: "", voicemail_mail_to: "", forward_destinations: "", enabled: true });
    setOpen(true);
  };

  const startEdit = (row: any) => {
    setEditing(row);
    const dests = row.forward_destinations || row.destinations || [];
    const forwardStr = Array.isArray(dests) ? dests.map((d: any) => d.voicemail_uuid_copy || d.voicemail_id || d).join(", ") : "";
    setForm({
      voicemail_id: row.voicemail_id || row.mailbox || "",
      voicemail_password: row.voicemail_password || row.password || "",
      description: row.description || row.voicemail_description || "",
      greeting_id: row.greeting_id != null ? String(row.greeting_id) : "",
      toll_allow: row.toll_allow || "",
      voicemail_mail_to: row.voicemail_mail_to || "",
      forward_destinations: forwardStr,
      enabled: row.enabled ?? row.voicemail_enabled ?? true,
    });
    setOpen(true);
  };

  const createOrUpdate = async () => {
    try {
      const payload: any = {
        voicemail_id: form.voicemail_id || null,
        voicemail_password: form.voicemail_password || null,
        enabled: form.enabled,
        description: form.description || null,
        greeting_id: form.greeting_id ? Number(form.greeting_id) : null,
        toll_allow: form.toll_allow || null,
        voicemail_mail_to: form.voicemail_mail_to || null,
        mailbox: form.voicemail_id || null,
        password: form.voicemail_password || null,
      };
      if (form.forward_destinations.trim()) {
        const dests = form.forward_destinations.split(",").map((s) => s.trim()).filter(Boolean);
        payload.forward_destinations = dests;
        payload.destinations = dests;
      }
      if (editing) {
        const uid = editing.voicemail_uuid || editing.id || editing.uuid;
        await apiFetch(`/voicemails/${uid}`, { method: "PATCH", body: JSON.stringify(payload) });
      } else {
        await apiFetch("/voicemails", { method: "POST", body: JSON.stringify(payload) });
      }
      setOpen(false); setEditing(null); load();
    } catch (e: any) { alert(e.message); }
  };

  const del = async (row: any) => {
    const uid = row.voicemail_uuid || row.id || row.uuid;
    const label = row.voicemail_id || row.mailbox || uid;
    if (!confirm(`Delete voicemail ${label}?`)) return;
    try { await apiFetch(`/voicemails/${uid}`, { method: "DELETE" }); load(); } catch (e: any) { alert(e.message); }
  };

  return (
    <div className="p-4 space-y-4">
      <div className="flex justify-between items-center"><h1 className="text-2xl font-bold">Voicemails</h1><div className="flex gap-2"><Badge>{total} total</Badge><Button onClick={startCreate}>Add Voicemail</Button></div></div>

      <div className="flex gap-2">
        <Input value={q} onChange={(e) => setQ(e.target.value)} placeholder="Search mailbox, description..." className="flex-1" />
        <Button variant="outline" onClick={() => { setPage(1); load(); }}>Search</Button>
      </div>

      <Card><CardHeader><CardTitle>Voicemails — mailboxes</CardTitle></CardHeader><CardContent>
          <div className="hidden md:block"><Table><TableHeader><TableRow><TableHead>Mailbox</TableHead><TableHead>Description</TableHead><TableHead>Greeting</TableHead><TableHead>Toll Allow</TableHead><TableHead>Forward</TableHead><TableHead>Enabled</TableHead><TableHead>Actions</TableHead></TableRow></TableHeader><TableBody>
                {data.map((d: any) => {
                  const uid = d.voicemail_uuid || d.id;
                  const mailbox = d.voicemail_id || d.mailbox || "-";
                  const dests = d.forward_destinations || d.destinations || [];
                  const forwardStr = Array.isArray(dests) ? dests.map((x: any) => x.voicemail_uuid_copy || x.voicemail_id || x).join(", ") : "";
                  return (
                    <TableRow key={String(uid)}>
                      <TableCell className="font-mono font-bold">{mailbox}</TableCell>
                      <TableCell className="text-xs max-w-[150px] truncate" title={d.description}>{d.description || "-"}</TableCell>
                      <TableCell>{d.greeting_id ?? "-"}</TableCell>
                      <TableCell className="text-xs">{d.toll_allow || "-"}</TableCell>
                      <TableCell className="font-mono text-xs max-w-[120px] truncate" title={forwardStr}>{forwardStr || "-"}</TableCell>
                      <TableCell>{(d.enabled ?? true) ? <Badge>enabled</Badge> : <Badge className="bg-muted text-muted-foreground">disabled</Badge>}</TableCell>
                      <TableCell><div className="flex gap-1"><Button variant="ghost" aria-label="Edit" title="Edit" className="h-9 w-9 p-0" onClick={() => startEdit(d)}><Pencil className="h-4 w-4" /></Button><Button variant="ghost" aria-label="Delete" title="Delete" className="h-9 w-9 p-0 text-destructive hover:text-destructive hover:bg-destructive/10" onClick={() => del(d)}><Trash2 className="h-4 w-4" /></Button></div></TableCell>
                    </TableRow>
                  );
                })}
                {data.length===0 && <TableRow><TableCell colSpan={7} className="text-center text-muted-foreground py-8">No voicemails — create via Add button</TableCell></TableRow>}
              </TableBody></Table></div>
          <div className="md:hidden grid gap-3">
            {data.map((d: any) => {
              const uid = d.voicemail_uuid || d.id;
              const mailbox = d.voicemail_id || d.mailbox;
              const dests = d.forward_destinations || d.destinations || [];
              const forwardStr = Array.isArray(dests) ? dests.map((x: any) => x.voicemail_uuid_copy || x).join(", ") : "";
              return (
                <Card key={String(uid)} className="p-3 space-y-1">
                  <div className="flex justify-between"><div className="font-mono font-bold">{mailbox}</div><Badge>{(d.enabled ?? true) ? "enabled" : "disabled"}</Badge></div>
                  <div className="text-xs">{d.description || "no description"}</div>
                  {forwardStr && <div className="font-mono text-xs text-muted-foreground">→ {forwardStr}</div>}
                  <div className="flex gap-2 mt-2"><Button variant="ghost" aria-label="Edit" title="Edit" className="flex-1 h-9" onClick={() => startEdit(d)}><Pencil className="h-4 w-4" /></Button><Button variant="ghost" aria-label="Delete" title="Delete" className="flex-1 h-9 text-destructive hover:text-destructive hover:bg-destructive/10" onClick={() => del(d)}><Trash2 className="h-4 w-4" /></Button></div>
                </Card>
              );
            })}
            {data.length===0 && <div className="text-center text-muted-foreground py-4">No voicemails</div>}
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
      <Dialog open={open} onOpenChange={(v) => { setOpen(v); if (!v) setEditing(null); }}><DialogHeader><DialogTitle>{editing ? "Edit Voicemail" : "Add Voicemail"}</DialogTitle></DialogHeader>
        <div className="space-y-3">
          <div className="grid grid-cols-2 gap-2">
            <div><Label>Mailbox * (2-7 digits)</Label><Input value={form.voicemail_id} onChange={(e) => setForm({ ...form, voicemail_id: e.target.value })} placeholder="1001" /></div>
            <div><Label>Password/PIN *</Label><Input type="password" value={form.voicemail_password} onChange={(e) => setForm({ ...form, voicemail_password: e.target.value })} placeholder="1234" /></div>
          </div>
          <div className="grid grid-cols-2 gap-2">
            <div><Label>Description</Label><Input value={form.description} onChange={(e) => setForm({ ...form, description: e.target.value })} placeholder="Sales voicemail" /></div>
            <div><Label>Greeting ID</Label><Input value={form.greeting_id} onChange={(e) => setForm({ ...form, greeting_id: e.target.value })} placeholder="1" /></div>
          </div>
          <div className="grid grid-cols-2 gap-2">
            <div><Label>Toll Allow</Label><Input value={form.toll_allow} onChange={(e) => setForm({ ...form, toll_allow: e.target.value })} placeholder="domestic" /></div>
            <div><Label>Mail To</Label><Input value={form.voicemail_mail_to} onChange={(e) => setForm({ ...form, voicemail_mail_to: e.target.value })} placeholder="user@example.com" /></div>
          </div>
          <div><Label>Forward Destinations (comma separated)</Label><Input value={form.forward_destinations} onChange={(e) => setForm({ ...form, forward_destinations: e.target.value })} placeholder="1002,1003" /></div>
          <div className="flex items-center gap-2"><input type="checkbox" id="vm-enabled" checked={form.enabled} onChange={(e) => setForm({ ...form, enabled: e.target.checked })} className="h-4 w-4" /><Label htmlFor="vm-enabled" className="cursor-pointer">Enabled</Label></div>
          <Button onClick={createOrUpdate} className="w-full">{editing ? "Save" : "Create"}</Button>
        </div></Dialog>
    </div>
  );
}
