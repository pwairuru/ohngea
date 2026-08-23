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

export default function TimeConditionsPlugin() {
  const confirm = useConfirm();
  const [items, setItems] = React.useState<any[]>([]);
  const [total, setTotal] = React.useState(0);
  const [page, setPage] = React.useState(1);
  const [q, setQ] = React.useState("");
  const [form, setForm] = React.useState({
    name: "business-hours",
    extension: "800",
    condition: "09:00-17:00 Mon-Fri",
    true_dest: "1000",
    false_dest: "voicemail",
    description: "",
    timezone: "",
    enabled: true,
  });
  const [error, setError] = React.useState<string | null>(null);

  const load = React.useCallback(() => {
    const params = new URLSearchParams({ page: String(page), per_page: "20" });
    if (q) params.set("q", q);
    apiFetch(`/time_conditions?${params.toString()}`)
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

  const create = async () => {
    setError(null);
    try {
      // ensure we have domain_id for backend (fallback: fetch first domain)
      let domain_id: string | null = null;
      try {
        const domains = await apiFetch("/domains");
        const list = Array.isArray(domains) ? domains : domains.data || [];
        domain_id = list[0]?.domain_id || list[0]?.domain_uuid || null;
      } catch {}
      const payload: any = {
        name: form.name,
        extension: form.extension,
        condition: form.condition,
        true_destination: form.true_dest,
        false_destination: form.false_dest,
        true_dest: form.true_dest,
        false_dest: form.false_dest,
        description: form.description || null,
        timezone: form.timezone || null,
        enabled: form.enabled,
      };
      if (domain_id) payload.domain_id = domain_id;
      await apiFetch("/time_conditions", { method: "POST", body: JSON.stringify(payload) });
      load();
    } catch (e: any) {
      setError(e.message);
    }
  };

  const remove = async (id: string) => {
    const ok = await confirm({ title: "Delete time condition?", confirmLabel: "Delete" });
    if (!ok) return;
    try {
      await apiFetch(`/time_conditions/${id}`, { method: "DELETE" });
      load();
    } catch (e: any) {
      setError(e.message);
    }
  };

  return (
    <div className="p-4 space-y-4">
      <h1 className="text-2xl font-bold">
        Time Conditions <Badge>plugin</Badge>
      </h1>
      <Card>
        <CardHeader>
          <CardTitle>Calendar / Time-based routing</CardTitle>
        </CardHeader>
        <CardContent className="text-sm text-muted-foreground">
          Conditions <code>09:00-17:00</code> map to dialplan <code>condition</code> <code>date_time</code> →{" "}
          <code>transfer</code> true/false dest. Real API: <code>GET /api/v1/time_conditions</code> with RLS, pagination
          (Link/X-Total-Count), HATEOAS, filtering.
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Add Condition</CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
            <div>
              <Label>Name *</Label>
              <Input value={form.name} onChange={(e) => setForm({ ...form, name: e.target.value })} placeholder="business-hours" />
            </div>
            <div>
              <Label>Extension * (2-7 digits)</Label>
              <Input value={form.extension} onChange={(e) => setForm({ ...form, extension: e.target.value })} placeholder="800" pattern="\d{2,7}" />
            </div>
          </div>
          <div>
            <Label>Condition (cron-like)</Label>
            <Input
              value={form.condition}
              onChange={(e) => setForm({ ...form, condition: e.target.value })}
              placeholder="09:00-17:00 Mon-Fri or minute-of-day 540-1020"
            />
          </div>
          <div className="grid grid-cols-2 gap-2">
            <div>
              <Label>True Dest</Label>
              <Input value={form.true_dest} onChange={(e) => setForm({ ...form, true_dest: e.target.value })} placeholder="1000" />
            </div>
            <div>
              <Label>False Dest</Label>
              <Input value={form.false_dest} onChange={(e) => setForm({ ...form, false_dest: e.target.value })} placeholder="voicemail" />
            </div>
          </div>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
            <div>
              <Label>Description</Label>
              <Input value={form.description} onChange={(e) => setForm({ ...form, description: e.target.value })} placeholder="Business hours routing" />
            </div>
            <div>
              <Label>Timezone</Label>
              <Input value={form.timezone} onChange={(e) => setForm({ ...form, timezone: e.target.value })} placeholder="America/New_York" />
            </div>
          </div>
          <div className="flex items-center gap-2">
            <input type="checkbox" checked={form.enabled} onChange={(e) => setForm({ ...form, enabled: e.target.checked })} id="tc-enabled" />
            <Label htmlFor="tc-enabled">Enabled</Label>
          </div>
          {error && <div className="text-sm text-red-600 border border-red-200 bg-red-50 p-2 rounded">{error}</div>}
          <Button onClick={create} className="w-full">
            Create
          </Button>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle className="flex items-center justify-between">
            <span>Conditions — {total} total</span>
            <div className="flex gap-2">
              <Input value={q} onChange={(e) => setQ(e.target.value)} placeholder="Search name, extension..." className="w-48" />
              <Button variant="outline" onClick={() => { setPage(1); load(); }}>Search</Button>
            </div>
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="hidden md:block">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Name</TableHead>
                  <TableHead>Ext</TableHead>
                  <TableHead>Condition</TableHead>
                  <TableHead>True→False</TableHead>
                  <TableHead>Status</TableHead>
                  <TableHead>Actions</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {items.map((it) => {
                  const id = it.time_condition_uuid || it.id;
                  const name = it.name || it.time_condition_name || it.dialplan_name;
                  const ext = it.extension || it.time_condition_extension || it.dialplan_number;
                  const cond = it.condition || it.time_of_day || it.minute_of_day || "-";
                  const tdest = it.true_destination || it.true_dest || "-";
                  const fdest = it.false_destination || it.false_dest || "-";
                  const enabled = it.enabled ?? it.dialplan_enabled ?? true;
                  return (
                    <TableRow key={id}>
                      <TableCell className="font-medium">{name}</TableCell>
                      <TableCell className="font-mono">{ext}</TableCell>
                      <TableCell className="font-mono text-xs">{cond}</TableCell>
                      <TableCell className="text-xs">
                        {tdest} → {fdest}
                      </TableCell>
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
                    <TableCell colSpan={6}><EmptyState title="No conditions" /></TableCell>
                  </TableRow>
                )}
              </TableBody>
            </Table>
          </div>
          <div className="md:hidden grid gap-2">
            {items.map((it) => {
              const id = it.time_condition_uuid || it.id;
              const name = it.name || it.time_condition_name;
              const cond = it.condition || "-";
              const tdest = it.true_destination || it.true_dest;
              const fdest = it.false_destination || it.false_dest;
              return (
                <Card key={id} className="p-3">
                  <div className="font-bold">{name}</div>
                  <div className="font-mono text-xs">{cond}</div>
                  <div className="text-xs">
                    {tdest} → {fdest}
                  </div>
                  <Button variant="outline" className="mt-2 w-full h-8" onClick={() => remove(id)}>Delete</Button>
                </Card>
              );
            })}
            {items.length === 0 && <EmptyState title="No conditions" />}
          </div>
          <div className="flex items-center justify-between mt-4 text-sm">
            <span>Page {page} — {items.length} items</span>
            <div className="flex gap-2">
              <Button disabled={page <= 1} onClick={() => setPage((p) => Math.max(1, p - 1))}>Prev</Button>
              <Button onClick={() => setPage((p) => p + 1)}>Next</Button>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
