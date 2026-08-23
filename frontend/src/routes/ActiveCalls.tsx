import * as React from "react";
import { DataTable } from "../components/DataTable";
import type { Column } from "../components/types";
import { PageHeader } from "../components/shell/PageHeader";

export function ActiveCalls() {
  const columns: Column<any>[] = [
    { key: "call_uuid", header: "Call UUID", className: "font-mono text-xs", cell: (r: any) => <span className="font-mono text-xs break-all">{r.call_uuid ?? r.uuid ?? "—"}</span> },
    { key: "direction", header: "Dir", cell: (r: any) => r.direction ?? "—" },
    { key: "cid_num", header: "Caller", className: "num", cell: (r: any) => r.cid_num ?? r.caller_id_number ?? "—" },
    { key: "dest", header: "Destination", className: "num", cell: (r: any) => r.dest ?? r.destination_number ?? "—" },
    { key: "state", header: "State", cell: (r: any) => r.state ?? r.callstate ?? "—" },
  ];
  return (
    <>
      <PageHeader title="Active Calls" description="Live active channels and bridged calls" />
      <DataTable resource="/active_calls" columns={columns} perPage={20} searchPlaceholder="Search active calls…" emptyTitle="No active calls" />
    </>
  );
}
