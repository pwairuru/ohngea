import * as React from "react";
import { DataTable } from "../components/DataTable";
import type { Column } from "../components/types";
import { PageHeader } from "../components/shell/PageHeader";

export function OperatorPanel() {
  const columns: Column<any>[] = [
    { key: "name", header: "Name", className: "num", cell: (r: any) => r.name ?? r.extension ?? JSON.stringify(r).slice(0,60) },
    { key: "details", header: "Details", cell: (r: any) => <span className="text-muted-foreground text-xs break-all">{JSON.stringify(r).slice(0, 80)}</span> },
  ];
  return (
    <>
      <PageHeader title="Operator Panel" description="Operator panel — extension presence and call state" />
      <DataTable resource="/operator_panel" columns={columns} perPage={20} searchPlaceholder="Search operator…" emptyTitle="No operator data" />
    </>
  );
}
