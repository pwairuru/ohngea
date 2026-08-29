import * as React from "react";
import { DataTable } from "../components/DataTable";
import type { Column } from "../components/types";
import { PageHeader } from "../components/shell/PageHeader";
import { Badge } from "../components/ui/badge";

export function CallCenterActive() {
  const columns: Column<any>[] = [
    { key: "queue_name", header: "Queue", className: "font-mono text-xs", cell: (r: any) => <span className="font-mono text-xs">{r.queue_name ?? r.name ?? "—"}</span> },
    { key: "queue_extension", header: "Extension", cell: (r: any) => r.queue_extension ?? r.extension ?? "—" },
    { key: "queue_strategy", header: "Strategy", cell: (r: any) => r.queue_strategy ?? r.strategy ?? "—" },
    { key: "status", header: "Source", cell: (r: any) => <Badge variant={r.source === "esl" || r.source === "esl_text" ? "default" : "secondary"}>{r.source ?? r.status ?? "DB"}</Badge> },
  ];
  return (
    <>
      <PageHeader title="Call Center Active" description="Live agent queues (ESL callcenter_config, fallback DB)" />
      <DataTable resource="/call_center_active" columns={columns} perPage={20} searchPlaceholder="Search queues…" emptyTitle="No active queues" emptyDescription="No active call center queues. ESL `callcenter_config queue list` empty." />
    </>
  );
}
