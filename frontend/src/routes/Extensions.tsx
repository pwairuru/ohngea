import * as React from "react";
import { apiFetch } from "../lib/api";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "../components/ui/table";
import { Card } from "../components/ui/card";
import { Button } from "../components/ui/button";

export function Extensions() {
  const [data, setData] = React.useState<any[]>([]);
  const [total, setTotal] = React.useState(0);
  const [q, setQ] = React.useState("");
  const [page, setPage] = React.useState(1);

  const load = React.useCallback(() => {
    apiFetch(`/extensions?page=${page}&per_page=10&q=${q}`).then((res) => {
      setData(res.data || []);
      setTotal(res.total || 0);
    }).catch(() => setData([]));
  }, [page, q]);

  React.useEffect(() => { load(); }, [load]);

  return (
    <div className="p-4 space-y-4">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold">Extensions</h1>
        <Button onClick={load}>Refresh</Button>
      </div>
      <div className="flex gap-2">
        <input value={q} onChange={(e) => setQ(e.target.value)} placeholder="Search extensions..." className="border rounded px-3 py-2 flex-1 min-h-[44px]" />
        <Button onClick={() => setPage(1)}>Search</Button>
      </div>

      {/* Desktop table */}
      <div className="hidden md:block">
        <Card>
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Extension</TableHead>
                <TableHead>Name</TableHead>
                <TableHead>Enabled</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {data.map((e) => (
                <TableRow key={e.extension_uuid}>
                  <TableCell>{e.extension}</TableCell>
                  <TableCell>{e.effective_caller_id_name}</TableCell>
                  <TableCell>{e.enabled ? "yes" : "no"}</TableCell>
                </TableRow>
              ))}
              {data.length === 0 && <TableRow><TableCell colSpan={3} className="text-center text-muted-foreground">No extensions</TableCell></TableRow>}
            </TableBody>
          </Table>
        </Card>
      </div>

      {/* Mobile cards */}
      <div className="grid gap-3 md:hidden" data-testid="extensions-cards">
        {data.map((e) => (
          <Card key={e.extension_uuid} data-card className="p-4">
            <div className="font-mono font-bold">{e.extension}</div>
            <div className="text-sm">{e.effective_caller_id_name}</div>
            <div className="text-xs text-muted-foreground">{e.enabled ? "enabled" : "disabled"}</div>
          </Card>
        ))}
        {data.length === 0 && <Card className="p-4 text-center text-muted-foreground">No extensions</Card>}
      </div>

      <div className="flex items-center justify-between text-sm">
        <span>Total: {total}</span>
        <div className="flex gap-2">
          <Button disabled={page <= 1} onClick={() => setPage((p) => Math.max(1, p - 1))}>Prev</Button>
          <span className="py-2">Page {page}</span>
          <Button onClick={() => setPage((p) => p + 1)}>Next</Button>
        </div>
      </div>
    </div>
  );
}
