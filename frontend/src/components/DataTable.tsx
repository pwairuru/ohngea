import * as React from "react";
import { apiFetch } from "../lib/api";
import { toastError } from "../lib/toast";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "./ui/table";
import { Card, CardContent } from "./ui/card";
import { Button } from "./ui/button";
import { Input } from "./ui/input";
import { Skeleton } from "./ui/skeleton";
import { EmptyState } from "./ui/empty-state";
import { ChevronLeft, ChevronRight, ArrowUpDown, Search } from "lucide-react";
import type { Column } from "./types";

type SortState = { key: string; order: "asc" | "desc" } | null;

export function DataTable<T extends Record<string, any>>({
  resource, columns, rowKey, rowActions, perPage = 20,
  emptyTitle = "Nothing here yet", emptyDescription, emptyIcon, emptyAction,
  toolbarActions, searchPlaceholder = "Search…", reloadKey = 0,
}: {
  resource: string;
  columns: Column<T>[];
  rowKey?: (row: T) => string | number;
  rowActions?: (row: T) => React.ReactNode;
  perPage?: number;
  emptyTitle?: string;
  emptyDescription?: string;
  emptyIcon?: React.ComponentType<{ className?: string }>;
  emptyAction?: React.ReactNode;
  toolbarActions?: React.ReactNode;
  searchPlaceholder?: string;
  reloadKey?: number;
}) {
  const [qInput, setQInput] = React.useState("");
  const [q, setQ] = React.useState("");
  const [page, setPage] = React.useState(1);
  const [sort, setSort] = React.useState<SortState>(null);
  const [rows, setRows] = React.useState<T[]>([]);
  const [total, setTotal] = React.useState(0);
  const [loading, setLoading] = React.useState(true);

  React.useEffect(() => {
    const t = setTimeout(() => { setQ(qInput); setPage(1); }, 300);
    return () => clearTimeout(t);
  }, [qInput]);

  const url = React.useMemo(() => {
    let u = `${resource}?page=${page}&per_page=${perPage}&q=${encodeURIComponent(q)}`;
    if (sort) u += `&sort=${encodeURIComponent(sort.key)}&order=${sort.order}`;
    return u;
  }, [resource, page, perPage, q, sort]);

  React.useEffect(() => {
    let alive = true;
    setLoading(true);
    apiFetch(url)
      .then((res: any) => {
        if (!alive) return;
        setRows(res?.data ?? []);
        setTotal(res?.total ?? res?.data?.length ?? 0);
      })
      .catch((err) => {
        if (!alive) return;
        toastError(err);
        setRows([]);
      })
      .finally(() => alive && setLoading(false));
    return () => { alive = false; };
  }, [url, reloadKey]);

  const cycleSort = (key: string) =>
    setSort((s) => (s?.key !== key ? { key, order: "asc" } : s.order === "asc" ? { key, order: "desc" } : null));

  const pages = Math.max(1, Math.ceil(total / perPage));
  const cellText = (row: T, c: Column<T>) => (c.cell ? c.cell(row) : String(row[c.key] ?? "—"));

  return (
    <div className="space-y-3">
      <div className="flex flex-wrap items-center gap-2">
        <div className="relative flex-1 min-w-[200px] max-w-sm">
          <Search className="pointer-events-none absolute left-2.5 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
          <Input value={qInput} onChange={(e) => setQInput(e.target.value)} placeholder={searchPlaceholder} className="pl-8" />
        </div>
        {toolbarActions}
      </div>

      <div className="hidden md:block rounded-md border bg-card">
        <Table>
          <TableHeader>
            <TableRow>
              {columns.map((c) => (
                <TableHead key={c.key} className={c.className}>
                  {c.sortable ? (
                    <button onClick={() => cycleSort(c.key)} className="inline-flex items-center gap-1 hover:text-foreground">
                      {c.header}
                      <ArrowUpDown className={`h-3 w-3 ${sort?.key === c.key ? "text-primary" : "text-muted-foreground"}`} />
                    </button>
                  ) : c.header}
                </TableHead>
              ))}
              {rowActions && <TableHead className="w-10" />}
            </TableRow>
          </TableHeader>
          <TableBody>
            {loading && rows.length === 0 &&
              Array.from({ length: 5 }).map((_, i) => (
                <TableRow key={`sk-${i}`}>
                  {columns.map((c) => (
                    <TableCell key={c.key}><Skeleton className="h-4 w-full max-w-[160px]" /></TableCell>
                  ))}
                  {rowActions && <TableCell />}
                </TableRow>
              ))}
            {!loading && rows.length === 0 && (
              <TableRow>
                <TableCell colSpan={columns.length + (rowActions ? 1 : 0)}>
                  <EmptyState icon={emptyIcon} title={emptyTitle} description={emptyDescription} action={emptyAction} />
                </TableCell>
              </TableRow>
            )}
            {rows.map((row, i) => (
              <TableRow key={rowKey ? rowKey(row) : i}>
                {columns.map((c) => (
                  <TableCell key={c.key} className={c.className}>{cellText(row, c)}</TableCell>
                ))}
                {rowActions && <TableCell className="text-right">{rowActions(row)}</TableCell>}
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </div>

      <div className="grid gap-3 md:hidden" data-testid={resource === "/extensions" ? "extensions-cards" : undefined}>
        {rows.map((row, i) => (
          <Card key={rowKey ? rowKey(row) : i}>
            <CardContent className="flex items-start justify-between gap-2 p-3">
              <div className="min-w-0 space-y-0.5">
                {columns.slice(0, 2).map((c) => (
                  <div key={c.key} className={c.className ?? ""}>
                    <span className="mr-1.5 text-[11px] uppercase tracking-wide text-muted-foreground">{c.header}</span>
                    <span className="break-all text-sm">{cellText(row, c)}</span>
                  </div>
                ))}
              </div>
              {rowActions && <div className="shrink-0">{rowActions(row)}</div>}
            </CardContent>
          </Card>
        ))}
        {!loading && rows.length === 0 && <EmptyState icon={emptyIcon} title={emptyTitle} description={emptyDescription} action={emptyAction} />}
        {loading && rows.length === 0 && <Skeleton className="h-24 w-full" />}
      </div>

      <div className="flex items-center justify-between text-sm text-muted-foreground">
        <span className="num">
          {total === 0 ? "0" : `${(page - 1) * perPage + 1}–${Math.min(page * perPage, total)} of ${total}`}
        </span>
        <div className="flex items-center gap-2">
          <Button variant="outline" size="sm" disabled={page <= 1} onClick={() => setPage((p) => p - 1)} aria-label="Previous page">
            <ChevronLeft className="h-4 w-4" /> Prev
          </Button>
          <span className="num">Page {page} of {pages}</span>
          <Button variant="outline" size="sm" disabled={page >= pages} onClick={() => setPage((p) => p + 1)} aria-label="Next page">
            Next <ChevronRight className="h-4 w-4" />
          </Button>
        </div>
      </div>
    </div>
  );
}
