import * as React from "react";
import { DataTable } from "../components/DataTable";
import type { Column } from "../components/types";
import { PageHeader } from "../components/shell/PageHeader";

export function ActiveCalls() {
  const columns: Column<any>[] = [
    { key: "name", header: "Name", className: "num", cell: (r: any) => r.name ?? r.id ?? JSON.stringify(r).slice(0,60) },
    { key: "details", header: "Details", cell: (r: any) => <span className="text-muted-foreground text-xs break-all">{JSON.stringify(r).slice(0, 80)}</span> },
  ];
  return (
    <>
      <PageHeader title="Active Calls" description="Live active channels and bridged calls" />
      <DataTable resource="/active_calls" columns={columns} perPage={20} searchPlaceholder="Search active calls…" emptyTitle="No active calls" />
    </>
  );
}
