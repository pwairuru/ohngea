import * as React from "react";
import { apiFetch } from "../../lib/api";
import { Card, CardContent, CardHeader, CardTitle } from "../../components/ui/card";
import { Badge } from "../../components/ui/badge";
import { Button } from "../../components/ui/button";
import {Pencil, Trash2, ChevronsLeft, ChevronLeft, ChevronRight, ChevronsRight} from "lucide-react";
import { Input } from "../../components/ui/input";
import { Label } from "../../components/ui/label";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "../../components/ui/table";

export default function VoicemailPlugin() {
  const [items, setItems] = React.useState<any[]>([]);
  const [total, setTotal] = React.useState(0);
  const [page, setPage] = React.useState(1);
  const [q, setQ] = React.useState("");
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
  const [error, setError] = React.useState<string | null>(null);

  const load = React.useCallback(() => {
    const params = new URLSearchParams({ page: String(page), per_page: "20" });
    if (q) params.set("q", q);
    apiFetch(`/voicemails?${params.toString()}`)
      .then((r) => {
        setItems(r.data || []);
        setTotal(r.total ?? 0);
        setError(null);
      })
      .catch((e: any) => {
        setError(String(e.message || e));
        setItems([]);
      });
  }, [page, q]);

  React.useEffect(() => {
    load();
  }, [load]);

  const getDomainId = async (): Promise<string> => {
    const domains = await apiFetch("/domains");
    const list = Array.isArray(domains) ? domains : domains.data || domains;
    const first = Array.isArray(list) ? list[0] : null;
    const domain_id = first?.domain_id || first?.domain_uuid || first?.id;
    if (!domain_id) throw new Error("No domain found — create domain first");
    return domain_id;
  };

  const create = async () => {
    try {
      setError(null);
      const domain_id = await getDomainId();
      const payload: any = {
        domain_id,
        voicemail_id: form.voicemail_id,
        voicemail_password: form.voicemail_password,
        enabled: form.enabled,
        description: form.description || null,
        greeting_id: form.greeting_id ? Number(form.greeting_id) : null,
        toll_allow: form.toll_allow || null,
        voicemail_mail_to: form.voicemail_mail_to || null,
      };
      if (form.forward_destinations.trim()) {
        const dests = form.forward_destinations.split(",").map((s) => s.trim()).filter(Boolean);
        payload.forward_destinations = dests;
        payload.destinations = dests;
      }
      // legacy aliases for robustness
      payload.mailbox = form.voicemail_id;
      payload.password = form.voicemail_password;
      await apiFetch("/voicemails", {
        method: "POST",
        body: JSON.stringify(payload),
      });
      load();
    } catch (e: any) {
      setError(e.message);
    }
  };

  const remove = async (id: string) => {
    if (!confirm("Delete voicemail?")) return;
    try {
      await apiFetch(`/voicemails/${id}`, { method: "DELETE" });
      load();
    } catch (e: any) {
      setError(e.message);
    }
  };

  return (
    <div className="p-4 space-y-4">
      <h1 className="text-2xl font-bold">
        Voicemail <Badge>plugin</Badge>
      </h1>
      <Card>
        <CardHeader>
          <CardTitle>Mailboxes — per extension (RLS per domain)</CardTitle>
        </CardHeader>
        <CardContent className="text-sm text-muted-foreground">
          Voicemail boxes partitioned by <code>domain_id</code> with RLS. Extension-like <code>voicemail_id</code> 2-7 digits, PIN
          <code>voicemail_password</code>. Real API: <code>GET /api/v1/voicemails</code> pagination (Link/X-Total-Count), HATEOAS,
          <code>mod_voicemail</code> via <code>GET /api/internal/freeswitch/xml</code>.
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Add Mailbox</CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          <div className="grid md:grid-cols-3 gap-2">
            <div>
              <Label>Mailbox * (2-7 digits)</Label>
              <Input value={form.voicemail_id} onChange={(e) => setForm({ ...form, voicemail_id: e.target.value })} placeholder="1001" pattern="\d{2,7}" />
            </div>
            <div>
              <Label>Password/PIN * (2-20 digits)</Label>
              <Input value={form.voicemail_password} onChange={(e) => setForm({ ...form, voicemail_password: e.target.value })} placeholder="1234" type="password" />
            </div>
            <div>
              <Label>Greeting ID</Label>
              <Input value={form.greeting_id} onChange={(e) => setForm({ ...form, greeting_id: e.target.value })} placeholder="1 (empty = default)" />
            </div>
          </div>
          <div className="grid md:grid-cols-2 gap-2">
            <div>
              <Label>Description</Label>
              <Input value={form.description} onChange={(e) => setForm({ ...form, description: e.target.value })} placeholder="Sales voicemail" />
            </div>
            <div>
              <Label>Toll Allow</Label>
              <Input value={form.toll_allow} onChange={(e) => setForm({ ...form, toll_allow: e.target.value })} placeholder="domestic, international, local" />
            </div>
          </div>
          <div>
            <Label>Mail To (comma separated emails)</Label>
            <Input value={form.voicemail_mail_to} onChange={(e) => setForm({ ...form, voicemail_mail_to: e.target.value })} placeholder="user@example.com" />
          </div>
          <div>
            <Label>Forward Destinations (comma separated mailbox ids or UUIDs)</Label>
            <Input value={form.forward_destinations} onChange={(e) => setForm({ ...form, forward_destinations: e.target.value })} placeholder="1002,1003" />
          </div>
          <div className="flex items-center gap-2">
            <input type="checkbox" checked={form.enabled} onChange={(e) => setForm({ ...form, enabled: e.target.checked })} id="vm-enabled" />
            <Label htmlFor="vm-enabled">Enabled</Label>
          </div>
          {error && <div className="text-sm text-red-600 border border-red-200 bg-red-50 p-2 rounded">{error}</div>}
          <Button onClick={create} className="w-full">
            Create
          </Button>
        </CardContent>
      </Card>

      <div className="flex gap-2">
        <Input value={q} onChange={(e) => setQ(e.target.value)} placeholder="Search mailbox, description..." className="flex-1" />
        <Button variant="outline" onClick={() => { setPage(1); load(); }}>Search</Button>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Boxes — {total} total</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="hidden md:block">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Mailbox</TableHead>
                  <TableHead>Password</TableHead>
                  <TableHead>Description</TableHead>
                  <TableHead>Greeting</TableHead>
                  <TableHead>Toll Allow</TableHead>
                  <TableHead>Forward</TableHead>
                  <TableHead>Enabled</TableHead>
                  <TableHead>Actions</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {items.map((it) => {
                  const id = it.voicemail_uuid || it.id;
                  const mailbox = it.voicemail_id || it.voicemail_mailbox || it.mailbox || it.extension;
                  const pwd = it.voicemail_password || it.password || "••••";
                  const dests = it.forward_destinations || it.destinations || [];
                  const forwardStr = Array.isArray(dests) ? dests.map((d: any) => d.voicemail_uuid_copy || d.voicemail_id || d).join(", ") : "";
                  return (
                    <TableRow key={id}>
                      <TableCell className="font-mono">{mailbox}</TableCell>
                      <TableCell className="font-mono text-xs">{pwd}</TableCell>
                      <TableCell className="max-w-[150px] truncate" title={it.description || it.voicemail_description}>{it.description || it.voicemail_description || "-"}</TableCell>
                      <TableCell>{it.greeting_id ?? it.greeting ?? "-"}</TableCell>
                      <TableCell className="text-xs">{it.toll_allow || "-"}</TableCell>
                      <TableCell className="font-mono text-xs max-w-[150px] truncate" title={forwardStr}>{forwardStr || "-"}</TableCell>
                      <TableCell>{it.enabled ?? it.voicemail_enabled ? <Badge>enabled</Badge> : <Badge className="bg-muted text-muted-foreground">disabled</Badge>}</TableCell>
                      <TableCell>
                        <Button variant="ghost" aria-label="Delete" title="Delete" className="h-9 w-9 p-0 text-destructive hover:text-destructive hover:bg-destructive/10" onClick={() => remove(id)}><Trash2 className="h-4 w-4" /></Button>
                      </TableCell>
                    </TableRow>
                  );
                })}
                {items.length === 0 && (
                  <TableRow>
                    <TableCell colSpan={8} className="text-center py-6 text-muted-foreground">
                      No mailboxes — create one above or check RLS tenant
                    </TableCell>
                  </TableRow>
                )}
              </TableBody>
            </Table>
          </div>
          <div className="md:hidden grid gap-2">
            {items.map((it) => {
              const id = it.voicemail_uuid || it.id;
              const mailbox = it.voicemail_id || it.mailbox;
              const dests = it.forward_destinations || it.destinations || [];
              const forwardStr = Array.isArray(dests) ? dests.map((d: any) => d.voicemail_uuid_copy || d).join(", ") : "";
              return (
                <Card key={id} className="p-3 space-y-1">
                  <div className="font-mono font-bold">{mailbox} <span className="text-xs font-normal">{it.description}</span></div>
                  <div className="text-xs">PIN: {it.voicemail_password || "••••"} {it.toll_allow ? `• toll ${it.toll_allow}` : ""}</div>
                  {forwardStr && <div className="font-mono text-xs text-muted-foreground">→ {forwardStr}</div>}
                  <Button variant="ghost" aria-label="Delete" title="Delete" className="h-9 w-9 p-0 text-destructive hover:text-destructive hover:bg-destructive/10" onClick={() => remove(id)}><Trash2 className="h-4 w-4" /></Button>
                </Card>
              );
            })}
            {items.length === 0 && <div className="text-center py-4 text-muted-foreground">No mailboxes</div>}
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
    </div>
  );
}
