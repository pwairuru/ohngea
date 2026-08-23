import * as React from "react";
import { EmptyState } from "../../components/ui/empty-state";
import { toast, toastError } from "../../lib/toast";
import { apiFetch } from "../../lib/api";
import { Card, CardContent, CardHeader, CardTitle } from "../../components/ui/card";
import { Badge } from "../../components/ui/badge";
import { Button } from "../../components/ui/button";
import { Input } from "../../components/ui/input";
import { Label } from "../../components/ui/label";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "../../components/ui/table";

export default function ContactsPlugin() {
  const [items, setItems] = React.useState<any[]>([]);
  const [form, setForm] = React.useState({ name: "Alice", number: "1001", email: "alice@example.com" });
  const load = () => apiFetch("/contacts").then((r) => setItems(r.data || [])).catch(() => {});
  React.useEffect(() => { load(); }, []);
  const create = async () => { await apiFetch("/contacts", { method: "POST", body: JSON.stringify(form) }); load(); };
  return (
    <div className="p-4 space-y-4">
      <h1 className="text-2xl font-bold">Contacts <Badge>plugin</Badge></h1>
      <Card><CardHeader><CardTitle>Address book per tenant</CardTitle></CardHeader><CardContent className="text-sm text-muted-foreground">Contacts partitioned by domain_id, searchable `q`.</CardContent></Card>
      <Card><CardHeader><CardTitle>Add Contact</CardTitle></CardHeader><CardContent className="space-y-3">
        <div className="grid md:grid-cols-3 gap-2"><div><Label>Name</Label><Input value={form.name} onChange={(e) => setForm({ ...form, name: e.target.value })} /></div><div><Label>Number</Label><Input value={form.number} onChange={(e) => setForm({ ...form, number: e.target.value })} /></div><div><Label>Email</Label><Input value={form.email} onChange={(e) => setForm({ ...form, email: e.target.value })} /></div></div>
        <Button onClick={create} className="w-full">Create</Button>
      </CardContent></Card>
      <Card><CardHeader><CardTitle>Contacts</CardTitle></CardHeader><CardContent>
        <div className="hidden md:block"><Table><TableHeader><TableRow><TableHead>Name</TableHead><TableHead>Number</TableHead><TableHead>Email</TableHead></TableRow></TableHeader><TableBody>{items.map((it) => <TableRow key={it.id}><TableCell>{it.name}</TableCell><TableCell className="font-mono">{it.number}</TableCell><TableCell>{it.email || "-"}</TableCell></TableRow>)}{items.length===0 && <TableRow><TableCell colSpan={3}><EmptyState title="No contacts" /></TableCell></TableRow>}</TableBody></Table></div>
        <div className="md:hidden grid gap-2">{items.map((it) => <Card key={it.id} className="p-3"><div className="font-bold">{it.name}</div><div className="font-mono text-xs">{it.number}</div><div className="text-xs">{it.email}</div></Card>)}{items.length===0 && <EmptyState title="No contacts" />}</div>
      </CardContent></Card>
    </div>
  );
}
