import * as React from "react";
import { DataTable } from "../components/DataTable";
import type { Column } from "../components/types";
import { PageHeader } from "../components/shell/PageHeader";
import { Badge } from "../components/ui/badge";

export function FifoList() {
  const columns: Column<any>[] = [
    { key: "fifo_name", header: "FIFO", className: "font-mono text-xs", cell: (r: any) => <span className="font-mono text-xs">{r.fifo_name ?? r.name ?? "—"}</span> },
    { key: "fifo_extension", header: "Extension", cell: (r: any) => r.fifo_extension ?? "—" },
    { key: "caller_count", header: "Callers", cell: (r: any) => <span className="num">{r.caller_count ?? 0}</span> },
    { key: "consumer_count", header: "Consumers", cell: (r: any) => <span className="num">{r.consumer_count ?? 0}</span> },
    { key: "status", header: "Status", cell: (r: any) => <Badge variant={r.status === "active" ? "default" : "secondary"}>{r.status ?? "—"}</Badge> },
  ];
  return (
    <>
      <PageHeader title="FIFO List" description="Live FIFO queue status (ESL fifo list, fallback DB)" />
      <DataTable resource="/fifo_list" columns={columns} perPage={20} searchPlaceholder="Search fifo…" emptyTitle="No FIFO queues" emptyDescription="No FIFO queues. ESL `fifo list` empty." />
    </>
  );
}
