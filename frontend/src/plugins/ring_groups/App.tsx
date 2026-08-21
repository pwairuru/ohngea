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
  const [form, setForm] = React.useState({ name: "sales-ring", extension: "2000", strategy: "simultaneous", members: "1001,1002,1003" });
  const load = () => apiFetch("/ring_groups").then((r) => setItems(r.data || [])).catch(() => {});
  React.useEffect(() => { load(); }, []);
  const create = async () => {
    await apiFetch("/ring_groups", { method: "POST", body: JSON.stringify({ name: form.name, extension: form.extension, strategy: form.strategy, members: form.members.split(",").map((s) => s.trim()) }) });
    load();
  };
  return (
    <div className="p-4 space-y-4">
      <h1 className="text-2xl font-bold">Ring Groups <Badge>plugin</Badge></h1>
      <Card><CardHeader><CardTitle>Hunt Groups — parallel/sequential</CardTitle></CardHeader><CardContent className="text-sm text-muted-foreground">Ring groups expand to `dialplan` `bridge` with `strategy` — simultaneous, enterprise, rollover. Per tenant.</CardContent></Card>
      <Card><CardHeader><CardTitle>Add Ring Group</CardTitle></CardHeader><CardContent className="space-y-3">
        <div className="grid md:grid-cols-2 gap-2"><div><Label>Name</Label><Input value={form.name} onChange={(e) => setForm({ ...form, name: e.target.value })} /></div><div><Label>Extension</Label><Input value={form.extension} onChange={(e) => setForm({ ...form, extension: e.target.value })} /></div></div>
        <div><Label>Strategy</Label><Input value={form.strategy} onChange={(e) => setForm({ ...form, strategy: e.target.value })} placeholder="simultaneous" /></div>
        <div><Label>Members (comma)</Label><Input value={form.members} onChange={(e) => setForm({ ...form, members: e.target.value })} /></div>
        <Button onClick={create} className="w-full">Create</Button>
      </CardContent></Card>
      <Card><CardHeader><CardTitle>Groups</CardTitle></CardHeader><CardContent>
        <div className="hidden md:block"><Table><TableHeader><TableRow><TableHead>Name</TableHead><TableHead>Extension</TableHead><TableHead>Strategy</TableHead></TableRow></TableHeader><TableBody>{items.map((it) => <TableRow key={it.id}><TableCell>{it.name}</TableCell><TableCell className="font-mono">{it.extension}</TableCell><TableCell><Badge>{it.strategy}</Badge></TableCell></TableRow>)}{items.length===0 && <TableRow><TableCell colSpan={3} className="text-center py-6 text-muted-foreground">No ring groups</TableCell></TableRow>}</TableBody></Table></div>
        <div className="md:hidden grid gap-2">{items.map((it) => <Card key={it.id} className="p-3"><div className="font-bold">{it.name}</div><div className="font-mono text-xs">{it.extension} — {it.strategy}</div></Card>)}{items.length===0 && <div className="text-center py-4 text-muted-foreground">No groups</div>}</div>
      </CardContent></Card>
    </div>
  );
}
