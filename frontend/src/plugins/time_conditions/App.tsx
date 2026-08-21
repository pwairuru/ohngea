import * as React from "react";
import { apiFetch } from "../../lib/api";
import { Card, CardContent, CardHeader, CardTitle } from "../../components/ui/card";
import { Badge } from "../../components/ui/badge";
import { Button } from "../../components/ui/button";
import { Input } from "../../components/ui/input";
import { Label } from "../../components/ui/label";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "../../components/ui/table";

export default function TimeConditionsPlugin() {
  const [items, setItems] = React.useState<any[]>([]);
  const [form, setForm] = React.useState({ name: "business-hours", condition: "09:00-17:00 Mon-Fri", true_dest: "1000", false_dest: "voicemail" });
  const load = () => apiFetch("/time_conditions").then((r) => setItems(r.data || [])).catch(() => {});
  React.useEffect(() => { load(); }, []);
  const create = async () => {
    await apiFetch("/time_conditions", { method: "POST", body: JSON.stringify(form) });
    load();
  };
  return (
    <div className="p-4 space-y-4">
      <h1 className="text-2xl font-bold">Time Conditions <Badge>plugin</Badge></h1>
      <Card><CardHeader><CardTitle>Calendar / Time-based routing</CardTitle></CardHeader><CardContent className="text-sm text-muted-foreground">Conditions `09:00-17:00` map to dialplan `condition` `date_time` → `transfer` true/false dest.</CardContent></Card>
      <Card><CardHeader><CardTitle>Add Condition</CardTitle></CardHeader><CardContent className="space-y-3">
        <div><Label>Name</Label><Input value={form.name} onChange={(e) => setForm({ ...form, name: e.target.value })} /></div>
        <div><Label>Condition (cron-like)</Label><Input value={form.condition} onChange={(e) => setForm({ ...form, condition: e.target.value })} /></div>
        <div className="grid grid-cols-2 gap-2"><div><Label>True Dest</Label><Input value={form.true_dest} onChange={(e) => setForm({ ...form, true_dest: e.target.value })} /></div><div><Label>False Dest</Label><Input value={form.false_dest} onChange={(e) => setForm({ ...form, false_dest: e.target.value })} /></div></div>
        <Button onClick={create} className="w-full">Create</Button>
      </CardContent></Card>
      <Card><CardHeader><CardTitle>Conditions</CardTitle></CardHeader><CardContent>
        <div className="hidden md:block"><Table><TableHeader><TableRow><TableHead>Name</TableHead><TableHead>Condition</TableHead><TableHead>True→False</TableHead></TableRow></TableHeader><TableBody>{items.map((it) => <TableRow key={it.id}><TableCell>{it.name}</TableCell><TableCell className="font-mono text-xs">{it.condition}</TableCell><TableCell>{it.true_dest} → {it.false_dest}</TableCell></TableRow>)}{items.length===0 && <TableRow><TableCell colSpan={3} className="text-center py-6 text-muted-foreground">No conditions</TableCell></TableRow>}</TableBody></Table></div>
        <div className="md:hidden grid gap-2">{items.map((it) => <Card key={it.id} className="p-3"><div className="font-bold">{it.name}</div><div className="font-mono text-xs">{it.condition}</div><div className="text-xs">{it.true_dest} → {it.false_dest}</div></Card>)}{items.length===0 && <div className="text-center py-4 text-muted-foreground">No conditions</div>}</div>
      </CardContent></Card>
    </div>
  );
}
