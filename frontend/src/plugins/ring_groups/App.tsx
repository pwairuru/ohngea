import * as React from "react";
import { apiFetch } from "../../lib/api";
import { Card, CardContent, CardHeader, CardTitle } from "../../components/ui/card";
import { Badge } from "../../components/ui/badge";
import { Button } from "../../components/ui/button";
import { Input } from "../../components/ui/input";
import { Label } from "../../components/ui/label";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "../../components/ui/table";

export default function RingGroupsPlugin() {
  const [items, setItems] = React.useState<any[]>([]);
  const [total, setTotal] = React.useState(0);
  const [q, setQ] = React.useState("");
  const [page, setPage] = React.useState(1);
  const [form, setForm] = React.useState({ name: "sales-ring", extension: "2000", strategy: "simultaneous", timeout: 30, members: "1001,1002,1003" });
  const [error, setError] = React.useState<string | null>(null);

  const load = React.useCallback(() => {
    const params = new URLSearchParams({ page: String(page), per_page: "20" });
    if (q) params.set("q", q);
    apiFetch(`/ring_groups?${params.toString()}`)
      .then((r) => {
        setItems(r.data || []);
        setTotal(r.total || 0);
        setError(null);
      })
      .catch((e) => setError(String(e.message || e)));
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
      const members = form.members
        .split(",")
        .map((s) => s.trim())
        .filter(Boolean);
      await apiFetch("/ring_groups", {
        method: "POST",
        body: JSON.stringify({
          domain_id,
          name: form.name,
          extension: form.extension,
          strategy: form.strategy,
          timeout: Number(form.timeout) || 30,
          enabled: true,
          members,
        }),
      });
      load();
    } catch (e: any) {
      setError(e.message);
    }
  };

  const remove = async (id: string) => {
    try {
      await apiFetch(`/ring_groups/${id}`, { method: "DELETE" });
      load();
    } catch (e: any) {
      setError(e.message);
    }
  };

  return (
    <div className="p-4 space-y-4">
      <h1 className="text-2xl font-bold">Ring Groups <Badge>plugin</Badge></h1>
      <Card><CardHeader><CardTitle>Hunt Groups — parallel/sequential</CardTitle></CardHeader><CardContent className="text-sm text-muted-foreground">Ring groups expand to <code>dialplan</code> <code>bridge</code> with <code>strategy</code> — simultaneous, enterprise, sequence, rollover, random. Per tenant with RLS. Extension 2-7 digits.</CardContent></Card>
      {error && <div className="rounded border border-destructive bg-destructive/10 p-3 text-sm text-destructive">{error}</div>}
      <Card><CardHeader><CardTitle>Add Ring Group</CardTitle></CardHeader><CardContent className="space-y-3">
        <div className="grid md:grid-cols-3 gap-2">
          <div><Label>Name</Label><Input value={form.name} onChange={(e) => setForm({ ...form, name: e.target.value })} placeholder="sales-ring" /></div>
          <div><Label>Extension *</Label><Input value={form.extension} onChange={(e) => setForm({ ...form, extension: e.target.value })} placeholder="2000" pattern="\d{2,7}" /></div>
          <div><Label>Timeout</Label><Input type="number" value={form.timeout} onChange={(e) => setForm({ ...form, timeout: Number(e.target.value) })} min={1} max={999} /></div>
        </div>
        <div><Label>Strategy</Label><select value={form.strategy} onChange={(e) => setForm({ ...form, strategy: e.target.value })} className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm">
          <option value="simultaneous">simultaneous</option>
          <option value="enterprise">enterprise</option>
          <option value="sequence">sequence</option>
          <option value="rollover">rollover</option>
          <option value="random">random</option>
          <option value="hunt">hunt (alias sequence)</option>
        </select></div>
        <div><Label>Members (comma separated extensions)</Label><Input value={form.members} onChange={(e) => setForm({ ...form, members: e.target.value })} placeholder="1001,1002,1003" /></div>
        <Button onClick={create} className="w-full">Create</Button>
      </CardContent></Card>
      <div className="flex gap-2">
        <Input value={q} onChange={(e) => setQ(e.target.value)} placeholder="Search name, extension, strategy..." className="flex-1" />
        <Button variant="outline" onClick={() => { setPage(1); load(); }}>Search</Button>
      </div>
      <Card><CardHeader><CardTitle>Groups — {total} total</CardTitle></CardHeader><CardContent>
        <div className="hidden md:block"><Table><TableHeader><TableRow><TableHead>Name</TableHead><TableHead>Extension</TableHead><TableHead>Strategy</TableHead><TableHead>Timeout</TableHead><TableHead>Members</TableHead><TableHead>Enabled</TableHead><TableHead>Actions</TableHead></TableRow></TableHeader><TableBody>
          {items.map((it) => {
            const id = it.ring_group_uuid || it.id;
            const members = it.members || it.destinations || [];
            const memberStr = Array.isArray(members) ? members.map((m: any) => m.destination_number || m).join(", ") : "";
            return (
              <TableRow key={id}>
                <TableCell className="font-medium">{it.name || it.ring_group_name}</TableCell>
                <TableCell className="font-mono">{it.extension || it.ring_group_extension}</TableCell>
                <TableCell><Badge>{it.strategy || it.ring_group_strategy}</Badge></TableCell>
                <TableCell>{it.timeout ?? "-"}</TableCell>
                <TableCell className="font-mono text-xs max-w-[200px] truncate" title={memberStr}>{memberStr || "-"}</TableCell>
                <TableCell>{it.enabled ?? it.ring_group_enabled ? <Badge>enabled</Badge> : <Badge className="bg-muted text-muted-foreground">disabled</Badge>}</TableCell>
                <TableCell><Button variant="outline" className="h-8" onClick={() => remove(id)}>Delete</Button></TableCell>
              </TableRow>
            );
          })}
          {items.length === 0 && <TableRow><TableCell colSpan={7} className="text-center py-6 text-muted-foreground">No ring groups</TableCell></TableRow>}
        </TableBody></Table></div>
        <div className="md:hidden grid gap-2">
          {items.map((it) => {
            const id = it.ring_group_uuid || it.id;
            const members = it.members || it.destinations || [];
            const memberStr = Array.isArray(members) ? members.map((m: any) => m.destination_number || m).join(", ") : "";
            return (
              <Card key={id} className="p-3 space-y-1">
                <div className="font-bold">{it.name || it.ring_group_name} <span className="font-mono text-xs">({it.extension || it.ring_group_extension})</span></div>
                <div className="text-xs"><Badge>{it.strategy || it.ring_group_strategy}</Badge> timeout {it.timeout}</div>
                <div className="font-mono text-xs text-muted-foreground">{memberStr || "no members"}</div>
                <Button variant="outline" size="sm" className="w-full mt-2" onClick={() => remove(id)}>Delete</Button>
              </Card>
            );
          })}
          {items.length === 0 && <div className="text-center py-4 text-muted-foreground">No groups</div>}
        </div>
        <div className="flex items-center justify-between mt-4 text-sm">
          <span>Total: {total}</span>
          <div className="flex gap-2">
            <Button disabled={page <= 1} onClick={() => setPage((p) => Math.max(1, p - 1))}>Prev</Button>
            <span className="py-2 px-2">Page {page}</span>
            <Button onClick={() => setPage((p) => p + 1)} disabled={page * 20 >= total}>Next</Button>
          </div>
        </div>
      </CardContent></Card>
    </div>
  );
}
