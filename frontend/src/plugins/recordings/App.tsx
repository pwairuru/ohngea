import * as React from "react";
import { EmptyState } from "../../components/ui/empty-state";
import { toast, toastError } from "../../lib/toast";
import { apiFetch } from "../../lib/api";
import { Card, CardContent, CardHeader, CardTitle } from "../../components/ui/card";
import { Badge } from "../../components/ui/badge";
import { Button } from "../../components/ui/button";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "../../components/ui/table";

export default function RecordingsPlugin() {
  const [items, setItems] = React.useState<any[]>([]);
  const load = () => apiFetch("/recordings").then((r) => setItems(r.data || [])).catch(() => {});
  React.useEffect(() => { load(); }, []);
  return (
    <div className="p-4 space-y-4">
      <h1 className="text-2xl font-bold">Recordings <Badge>plugin</Badge></h1>
      <Card><CardHeader><CardTitle>Call recordings — S3/MinIO</CardTitle></CardHeader><CardContent className="text-sm text-muted-foreground">Recordings per tenant, partitioned, `GET /recordings` with `Link` pagination, playback via object store.</CardContent></Card>
      <Card><CardHeader><CardTitle>Recordings</CardTitle></CardHeader><CardContent>
        <div className="hidden md:block"><Table><TableHeader><TableRow><TableHead>File</TableHead><TableHead>Duration</TableHead><TableHead>Date</TableHead></TableRow></TableHeader><TableBody>{items.map((it) => <TableRow key={it.id}><TableCell className="font-mono text-xs">{it.file || it.name || "recording.wav"}</TableCell><TableCell>{it.duration || "-"}s</TableCell><TableCell className="text-xs">{it.created_at || "-"}</TableCell></TableRow>)}{items.length===0 && <TableRow><TableCell colSpan={3}><EmptyState title="No recordings" /></TableCell></TableRow>}</TableBody></Table></div>
        <div className="md:hidden grid gap-2">{items.map((it) => <Card key={it.id} className="p-3"><div className="font-mono text-xs">{it.file}</div><div className="text-xs">{it.duration}s</div></Card>)}{items.length===0 && <EmptyState title="No recordings" />}</div>
      </CardContent></Card>
      <Button variant="outline" className="w-full" onClick={load}>Refresh</Button>
    </div>
  );
}
