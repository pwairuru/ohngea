import * as React from "react";
import { apiFetch } from "../../lib/api";
import { Card, CardContent, CardHeader, CardTitle } from "../../components/ui/card";
import { Badge } from "../../components/ui/badge";
import { Button } from "../../components/ui/button";
import { Input } from "../../components/ui/input";
import { Label } from "../../components/ui/label";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "../../components/ui/table";

export default function VoicemailPlugin() {
  const [items, setItems] = React.useState<any[]>([]);
  const [form, setForm] = React.useState({ mailbox: "1001", greeting: "Unavailable" });
  const load = () => apiFetch("/voicemail").then((r) => setItems(r.data || [])).catch(() => {});
  React.useEffect(() => { load(); }, []);
  const create = async () => {
    await apiFetch("/voicemail", { method: "POST", body: JSON.stringify({ mailbox: form.mailbox, greeting: form.greeting }) });
    load();
  };
  return (
    <div className="p-4 space-y-4">
      <h1 className="text-2xl font-bold">Voicemail <Badge>plugin</Badge></h1>
      <Card><CardHeader><CardTitle>Mailboxes — per extension</CardTitle></CardHeader><CardContent className="text-sm text-muted-foreground">Voicemail boxes partitioned by domain_id, `mod_voicemail` via `GET /api/internal/freeswitch/xml` `configuration` section, storage S3/MinIO.</CardContent></Card>
      <Card><CardHeader><CardTitle>Add Mailbox</CardTitle></CardHeader><CardContent className="space-y-3">
        <div><Label>Mailbox (extension)</Label><Input value={form.mailbox} onChange={(e) => setForm({ ...form, mailbox: e.target.value })} placeholder="1001" /></div>
        <div><Label>Greeting</Label><Input value={form.greeting} onChange={(e) => setForm({ ...form, greeting: e.target.value })} /></div>
        <Button onClick={create} className="w-full">Create</Button>
      </CardContent></Card>
      <Card><CardHeader><CardTitle>Boxes</CardTitle></CardHeader><CardContent>
        <div className="hidden md:block"><Table><TableHeader><TableRow><TableHead>Mailbox</TableHead><TableHead>Greeting</TableHead></TableRow></TableHeader><TableBody>{items.map((it) => <TableRow key={it.id}><TableCell className="font-mono">{it.mailbox}</TableCell><TableCell>{it.greeting || "-"}</TableCell></TableRow>)}{items.length===0 && <TableRow><TableCell colSpan={2} className="text-center py-6 text-muted-foreground">No mailboxes</TableCell></TableRow>}</TableBody></Table></div>
        <div className="md:hidden grid gap-2">{items.map((it) => <Card key={it.id} className="p-3"><div className="font-mono font-bold">{it.mailbox}</div><div className="text-xs">{it.greeting}</div></Card>)}{items.length===0 && <div className="text-center py-4 text-muted-foreground">No mailboxes</div>}</div>
      </CardContent></Card>
    </div>
  );
}
