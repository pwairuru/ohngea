import * as React from "react";
import { apiFetch } from "../lib/api";
import { Card } from "../components/ui/card";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "../components/ui/table";

export function Cdr() {
  const [data, setData] = React.useState<any[]>([]);
  const [total, setTotal] = React.useState(0);
  React.useEffect(() => {
    apiFetch("/cdr?page=1&per_page=10").then((r) => { setData(r.data || []); setTotal(r.total || 0); }).catch(() => {});
  }, []);
  return (
    <div className="p-4 space-y-4">
      <h1 className="text-2xl font-bold">Call Detail Records</h1>
      <div className="hidden md:block">
        <Card>
          <Table>
            <TableHeader><TableRow><TableHead>Caller</TableHead><TableHead>Destination</TableHead><TableHead>Start</TableHead><TableHead>Duration</TableHead></TableRow></TableHeader>
            <TableBody>
              {data.map((c) => <TableRow key={c.cdr_uuid}><TableCell>{c.caller_id_number}</TableCell><TableCell>{c.destination_number}</TableCell><TableCell>{new Date(c.start_stamp).toLocaleString()}</TableCell><TableCell>{c.duration}s</TableCell></TableRow>)}
              {data.length===0 && <TableRow><TableCell colSpan={4} className="text-center">No CDRs</TableCell></TableRow>}
            </TableBody>
          </Table>
        </Card>
      </div>
      <div className="grid gap-3 md:hidden">
        {data.map((c) => <Card key={c.cdr_uuid} className="p-3"><div className="font-mono">{c.caller_id_number} → {c.destination_number}</div><div className="text-xs text-muted-foreground">{c.start_stamp}</div></Card>)}
        {data.length===0 && <Card className="p-4 text-center">No CDRs — total {total}</Card>}
      </div>
    </div>
  );
}
