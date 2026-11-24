import * as React from "react";
import { apiFetch } from "../lib/api";
import { PageHeader } from "./shell/PageHeader";
import { Button } from "./ui/button";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "./ui/table";
import { Card, CardContent } from "./ui/card";
import { Skeleton } from "./ui/skeleton";
import { EmptyState } from "./ui/empty-state";
import { RefreshCw } from "lucide-react";
import { toastError } from "../lib/toast";

export function LiveTable({ title, description, resource, refreshSecs = 30, rowActions }: {
  title: string;
  description: string;
  resource: string;
  refreshSecs?: number;
  rowActions?: (row: any, reload: () => void) => React.ReactNode;
}) {
  const [rows, setRows] = React.useState<any[]>([]);
  const [total, setTotal] = React.useState(0);
  const [loading, setLoading] = React.useState(true);
  const [tick, setTick] = React.useState(0);
  const reload = () => setTick((t) => t + 1);

  React.useEffect(() => {
    let alive = true;
    setLoading(true);
    apiFetch(`${resource}?per_page=100`)
      .then((res: any) => {
        if (!alive) return;
        setRows(res?.data ?? []);
        setTotal(res?.total ?? res?.data?.length ?? 0);
      })
      .catch((err) => { if (alive) { toastError(err); setRows([]); } })
      .finally(() => alive && setLoading(false));
    return () => { alive = false; };
  }, [resource, tick]);

  React.useEffect(() => {
    if (!refreshSecs) return;
    const t = setInterval(() => setTick((x) => x + 1), refreshSecs * 1000);
    return () => clearInterval(t);
  }, [refreshSecs, resource]);

  const columns = React.useMemo(() => {
    const keys: string[] = [];
    for (const r of rows.slice(0, 20)) {
      for (const k of Object.keys(r ?? {})) {
        if (k.startsWith("_") || k.endsWith("_uuid") || k === "domain_id") continue;
        if (!keys.includes(k)) keys.push(k);
      }
    }
    return keys.slice(0, 6);
  }, [rows]);

  const cell = (v: any) => {
    if (v === null || v === undefined || v === "") return "—";
    if (typeof v === "boolean") return v ? "yes" : "no";
    const s = String(v);
    return s.length > 50 ? s.slice(0, 50) + "…" : s;
  };

  return (
    <>
      <PageHeader
        title={title}
        description={`${description} · ${total} live · auto-refresh ${refreshSecs}s`}
        actions={<Button variant="outline" onClick={reload}><RefreshCw className="h-4 w-4" /> Refresh</Button>}
      />
      <div className="hidden md:block rounded-md border bg-card">
        <Table>
          <TableHeader>
            <TableRow>
              {columns.map((c) => <TableHead key={c}>{c.replace(/_/g, " ")}</TableHead>)}
              {rowActions && <TableHead className="w-10" />}
            </TableRow>
          </TableHeader>
          <TableBody>
            {loading && rows.length === 0 && (
              <TableRow>{columns.map((c) => <TableCell key={c}><Skeleton className="h-4 w-full max-w-[160px]" /></TableCell>)}{rowActions && <TableCell />}</TableRow>
            )}
            {!loading && rows.length === 0 && (
              <TableRow><TableCell colSpan={columns.length + 1}><EmptyState title={`No ${title.toLowerCase()}`} description="Nothing live right now" /></TableCell></TableRow>
            )}
            {rows.map((row: any, i: number) => (
              <TableRow key={row.call_uuid ?? row.uuid ?? row.id ?? i}>
                {columns.map((c) => <TableCell key={c} className="font-mono text-xs">{cell(row[c])}</TableCell>)}
                {rowActions && <TableCell className="text-right">{rowActions(row, reload)}</TableCell>}
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </div>
      <div className="grid gap-3 md:hidden">
        {rows.map((row: any, i: number) => (
          <Card key={row.call_uuid ?? row.uuid ?? row.id ?? i}>
            <CardContent className="p-3 space-y-0.5">
              {columns.slice(0, 3).map((c) => (
                <div key={c}><span className="mr-1.5 text-[11px] uppercase tracking-wide text-muted-foreground">{c.replace(/_/g, " ")}</span><span className="break-all font-mono text-sm">{cell(row[c])}</span></div>
              ))}
              {rowActions && <div className="pt-1">{rowActions(row, reload)}</div>}
            </CardContent>
          </Card>
        ))}
        {!loading && rows.length === 0 && <EmptyState title={`No ${title.toLowerCase()}`} description="Nothing live right now" />}
      </div>
    </>
  );
}
