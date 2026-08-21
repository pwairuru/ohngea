import * as React from "react";
import { apiFetch } from "../../lib/api";
import { Card, CardContent, CardHeader, CardTitle } from "../../components/ui/card";
import { Button } from "../../components/ui/button";
import { Input } from "../../components/ui/input";
import { Label } from "../../components/ui/label";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "../../components/ui/table";
import { Badge } from "../../components/ui/badge";

export default function FaxPlugin() {
  const [items, setItems] = React.useState<any[]>([]);
  const [form, setForm] = React.useState({ destination: "15551234567", file: "" });
  const load = () => apiFetch("/fax").then((r) => setItems(r.data || [])).catch(() => {});
  React.useEffect(() => { load(); }, []);
  const send = async () => {
    await apiFetch("/fax", { method: "POST", body: JSON.stringify({ destination: form.destination, status: "queued", created_at: new Date().toISOString() }) });
    load();
  };
  return (
    <div className="p-4 space-y-4">
      <h1 className="text-2xl font-bold">Fax <Badge>plugin</Badge> <Badge variant="outline">MIT</Badge></h1>
      <Card><CardHeader><CardTitle>Fax Queue — Celery + Redis</CardTitle></CardHeader><CardContent className="text-sm text-muted-foreground">Per-tenant fax queue, worker `app/workers/celery_app.py` sends via `mod_spandsp`, `POST /api/v1/fax` queues, `GET /fax` lists partitioned by domain_id.</CardContent></Card>
      <Card><CardHeader><CardTitle>Send Fax</CardTitle></CardHeader><CardContent className="space-y-3">
        <div><Label>Destination</Label><Input value={form.destination} onChange={(e) => setForm({ ...form, destination: e.target.value })} placeholder="15551234567" /></div>
        <div><Label>File (placeholder)</Label><Input value={form.file} onChange={(e) => setForm({ ...form, file: e.target.value })} placeholder="path/to.pdf" /></div>
        <Button onClick={send} className="w-full">Queue Fax</Button>
      </CardContent></Card>
      <Card><CardHeader><CardTitle>Queue</CardTitle></CardHeader><CardContent>
        <div className="hidden md:block"><Table><TableHeader><TableRow><TableHead>Destination</TableHead><TableHead>Status</TableHead></TableRow></TableHeader><TableBody>{items.map((it) => <TableRow key={it.id}><TableCell className="font-mono">{it.destination}</TableCell><TableCell><Badge>{it.status || "queued"}</Badge></TableCell></TableRow>)}{items.length===0 && <TableRow><TableCell colSpan={2} className="text-center text-muted-foreground py-6">No faxes</TableCell></TableRow>}</TableBody></Table></div>
        <div className="md:hidden grid gap-2">{items.map((it) => <Card key={it.id} className="p-3"><div className="font-mono">{it.destination}</div><Badge>{it.status}</Badge></Card>)}{items.length===0 && <div className="text-center text-muted-foreground py-4">No faxes</div>}</div>
      </CardContent></Card>
    </div>
  );
}
