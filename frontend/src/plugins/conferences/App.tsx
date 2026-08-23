import * as React from "react";
import { EmptyState } from "../../components/ui/empty-state";
import { toast, toastError } from "../../lib/toast";
import { useConfirm } from "../../components/ui/confirm";
import { apiFetch } from "../../lib/api";
import { Card, CardContent, CardHeader, CardTitle } from "../../components/ui/card";
import { Badge } from "../../components/ui/badge";
import { Button } from "../../components/ui/button";
import { Input } from "../../components/ui/input";
import { Label } from "../../components/ui/label";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "../../components/ui/table";

export default function ConferencesPlugin() {
  const confirm = useConfirm();
  const [items, setItems] = React.useState<any[]>([]);
  const [total, setTotal] = React.useState(0);
  const [page, setPage] = React.useState(1);
  const [q, setQ] = React.useState("");
  const [form, setForm] = React.useState({
    conference_name: "sales-conf",
    conference_extension: "3000",
    conference_pin_number: "1234",
    conference_profile: "default",
    conference_flags: "",
    conference_enabled: true,
    conference_description: "",
    conference_order: "",
  });
  const [error, setError] = React.useState<string | null>(null);

  const load = React.useCallback(() => {
    const params = new URLSearchParams({ page: String(page), per_page: "20" });
    if (q) params.set("q", q);
    apiFetch(`/conferences?${params.toString()}`)
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
        conference_name: form.conference_name,
        conference_extension: form.conference_extension,
        conference_pin_number: form.conference_pin_number || null,
        conference_profile: form.conference_profile || "default",
        conference_enabled: form.conference_enabled,
        conference_description: form.conference_description || null,
        conference_flags: form.conference_flags || null,
        // legacy/alias fields for robustness
        name: form.conference_name,
        extension: form.conference_extension,
        pin_number: form.conference_pin_number,
        profile: form.conference_profile,
        enabled: form.conference_enabled,
        description: form.conference_description,
      };
      if (form.conference_order.trim()) {
        try {
          payload.conference_order = Number(form.conference_order);
          payload.order = Number(form.conference_order);
        } catch {}
      }
      await apiFetch("/conferences", {
        method: "POST",
        body: JSON.stringify(payload),
      });
      load();
    } catch (e: any) {
      setError(e.message);
    }
  };

  const remove = async (id: string) => {
    const ok = await confirm({ title: "Delete conference?", confirmLabel: "Delete" });
    if (!ok) return;
    try {
      await apiFetch(`/conferences/${id}`, { method: "DELETE" });
      load();
    } catch (e: any) {
      setError(e.message);
    }
  };

  return (
    <div className="p-4 space-y-4">
      <h1 className="text-2xl font-bold">
        Conferences <Badge>plugin</Badge>
      </h1>
      <Card>
        <CardHeader>
          <CardTitle>Conference bridges — mod_conference</CardTitle>
        </CardHeader>
        <CardContent className="text-sm text-muted-foreground">
          Per-tenant conference rooms, PIN, <code>bridge</code> to <code>conference:{"{name}"}@default</code> — profile <code>default</code> / <code>wideband</code> etc.
          Real API: <code>GET /api/v1/conferences</code> pagination (Link/X-Total-Count), HATEOAS, RLS.
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Add Conference</CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          <div className="grid md:grid-cols-3 gap-2">
            <div>
              <Label>Name *</Label>
              <Input value={form.conference_name} onChange={(e) => setForm({ ...form, conference_name: e.target.value })} placeholder="sales-conf" />
            </div>
            <div>
              <Label>Extension * (2-7 digits)</Label>
              <Input value={form.conference_extension} onChange={(e) => setForm({ ...form, conference_extension: e.target.value })} placeholder="3000" pattern="\d{2,7}" />
            </div>
            <div>
              <Label>PIN</Label>
              <Input value={form.conference_pin_number} onChange={(e) => setForm({ ...form, conference_pin_number: e.target.value })} placeholder="1234" />
            </div>
          </div>
          <div className="grid md:grid-cols-3 gap-2">
            <div>
              <Label>Profile</Label>
              <select value={form.conference_profile} onChange={(e) => setForm({ ...form, conference_profile: e.target.value })} className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm">
                <option value="default">default</option>
                <option value="wideband">wideband</option>
                <option value="ultra-wideband">ultra-wideband</option>
                <option value="cdquality">cdquality</option>
              </select>
            </div>
            <div>
              <Label>Flags</Label>
              <Input value={form.conference_flags} onChange={(e) => setForm({ ...form, conference_flags: e.target.value })} placeholder="mute|deaf|waste|moderator" />
            </div>
            <div>
              <Label>Order</Label>
              <Input type="number" value={form.conference_order} onChange={(e) => setForm({ ...form, conference_order: e.target.value })} placeholder="100" />
            </div>
          </div>
          <div>
            <Label>Description</Label>
            <Input value={form.conference_description} onChange={(e) => setForm({ ...form, conference_description: e.target.value })} placeholder="Sales team conference" />
          </div>
          <div className="flex items-center gap-2">
            <input type="checkbox" checked={form.conference_enabled} onChange={(e) => setForm({ ...form, conference_enabled: e.target.checked })} id="conf-enabled" />
            <Label htmlFor="conf-enabled">Enabled</Label>
          </div>
          {error && <div className="text-sm text-red-600 border border-red-200 bg-red-50 p-2 rounded">{error}</div>}
          <Button onClick={create} className="w-full">
            Create
          </Button>
        </CardContent>
      </Card>

      <div className="flex gap-2">
        <Input value={q} onChange={(e) => setQ(e.target.value)} placeholder="Search name, extension, profile..." className="flex-1" />
        <Button variant="outline" onClick={() => { setPage(1); load(); }}>Search</Button>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Rooms — {total} total</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="hidden md:block">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Name</TableHead>
                  <TableHead>Extension</TableHead>
                  <TableHead>PIN</TableHead>
                  <TableHead>Profile</TableHead>
                  <TableHead>Flags</TableHead>
                  <TableHead>Enabled</TableHead>
                  <TableHead>Actions</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {items.map((it) => {
                  const id = it.conference_uuid || it.id;
                  const name = it.conference_name || it.name || it.conferenceName;
                  const ext = it.conference_extension || it.extension || it.conferenceExtension;
                  const pin = it.conference_pin_number || it.pin_number || it.pin || "-";
                  const prof = it.conference_profile || it.profile || "default";
                  const flags = it.conference_flags || it.flags || "-";
                  const enabled = it.conference_enabled ?? it.enabled ?? true;
                  return (
                    <TableRow key={id}>
                      <TableCell className="font-medium">{name}</TableCell>
                      <TableCell className="font-mono">{ext}</TableCell>
                      <TableCell className="font-mono text-xs">{pin}</TableCell>
                      <TableCell><Badge>{prof}</Badge></TableCell>
                      <TableCell className="text-xs">{flags}</TableCell>
                      <TableCell>{enabled ? <Badge>enabled</Badge> : <Badge className="bg-muted text-muted-foreground">disabled</Badge>}</TableCell>
                      <TableCell>
                        <Button variant="outline" className="h-8" onClick={() => remove(id)}>
                          Delete
                        </Button>
                      </TableCell>
                    </TableRow>
                  );
                })}
                {items.length === 0 && (
                  <TableRow>
                    <TableCell colSpan={7}><EmptyState title="No conferences" /></TableCell>
                  </TableRow>
                )}
              </TableBody>
            </Table>
          </div>
          <div className="md:hidden grid gap-2">
            {items.map((it) => {
              const id = it.conference_uuid || it.id;
              const name = it.conference_name || it.name;
              const ext = it.conference_extension || it.extension;
              const pin = it.conference_pin_number || it.pin_number || it.pin;
              const prof = it.conference_profile || it.profile;
              return (
                <Card key={id} className="p-3 space-y-1">
                  <div className="font-bold">{name} <span className="font-mono text-xs">({ext})</span></div>
                  <div className="text-xs">PIN {pin || "-"} • {prof}</div>
                  <div className="text-xs text-muted-foreground">{it.conference_description || it.description || ""}</div>
                  <Button variant="outline" size="sm" className="w-full mt-2" onClick={() => remove(id)}>Delete</Button>
                </Card>
              );
            })}
            {items.length === 0 && <EmptyState title="No conferences" />}
          </div>
          <div className="flex items-center justify-between mt-4 text-sm">
            <span>Total: {total}</span>
            <div className="flex gap-2">
              <Button disabled={page <= 1} onClick={() => setPage((p) => Math.max(1, p - 1))}>Prev</Button>
              <span className="py-2 px-2">Page {page}</span>
              <Button onClick={() => setPage((p) => p + 1)} disabled={page * 20 >= total}>Next</Button>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
