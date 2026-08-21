import * as React from "react";
import { apiFetch } from "../../lib/api";
import { Card, CardContent, CardHeader, CardTitle } from "../../components/ui/card";
import { Badge } from "../../components/ui/badge";
import { Button } from "../../components/ui/button";
import { Input } from "../../components/ui/input";
import { Label } from "../../components/ui/label";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "../../components/ui/table";

export default function ConferencesPlugin() {
  const [items, setItems] = React.useState<any[]>([]);
  const [form, setForm] = React.useState({ name: "sales-conf", extension: "3000", pin: "1234" });
  const load = () => apiFetch("/conferences").then((r) => setItems(r.data || [])).catch(() => {});
  React.useEffect(() => { load(); }, []);
  const create = async () => { await apiFetch("/conferences", { method: "POST", body: JSON.stringify(form) }); load(); };
  return (
    <div className="p-4 space-y-4">
      <h1 className="text-2xl font-bold">Conferences <Badge>plugin</Badge></h1>
      <Card><CardHeader><CardTitle>Conference bridges — mod_conference</CardTitle></CardHeader><CardContent className="text-sm text-muted-foreground">Per-tenant conference rooms, PIN, `bridge` to `conference:{name}@default`.</CardContent></Card>
      <Card><CardHeader><CardTitle>Add Conference</CardTitle></CardHeader><CardContent className="space-y-3">
        <div className="grid md:grid-cols-3 gap-2"><div><Label>Name</Label><Input value={form.name} onChange={(e) => setForm({ ...form, name: e.target.value })} /></div><div><Label>Extension</Label><Input value={form.extension} onChange={(e) => setForm({ ...form, extension: e.target.value })} /></div><div><Label>PIN</Label><Input value={form.pin} onChange={(e) => setForm({ ...form, pin: e.target.value })} /></div></div>
        <Button onClick={create} className="w-full">Create</Button>
      </CardContent></Card>
      <Card><CardHeader><CardTitle>Rooms</CardTitle></CardHeader><CardContent>
        <div className="hidden md:block"><Table><TableHeader><TableRow><TableHead>Name</TableHead><TableHead>Extension</TableHead><TableHead>PIN</TableHead></TableRow></TableHeader><TableBody>{items.map((it) => <TableRow key={it.id}><TableCell>{it.name}</TableCell><TableCell className="font-mono">{it.extension}</TableCell><TableCell>{it.pin || "-"}</TableCell></TableRow>)}{items.length===0 && <TableRow><TableCell colSpan={3} className="text-center py-6 text-muted-foreground">No conferences</TableCell></TableRow>}</TableBody></Table></div>
        <div className="md:hidden grid gap-2">{items.map((it) => <Card key={it.id} className="p-3"><div className="font-bold">{it.name}</div><div className="font-mono text-xs">{it.extension} pin {it.pin}</div></Card>)}{items.length===0 && <div className="text-center py-4 text-muted-foreground">No conferences</div>}</div>
      </CardContent></Card>
    </div>
  );
}
