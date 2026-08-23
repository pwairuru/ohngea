import * as React from "react";
import { DataTable } from "../components/DataTable";
import type { Column } from "../components/types";
import { PageHeader } from "../components/shell/PageHeader";
import { Phone } from "lucide-react";

type CdrRow = {
  cdr_uuid: string;
  caller_id_number: string;
  destination_number: string;
  start_stamp: string;
  duration: number;
};

const columns: Column<CdrRow>[] = [
  { key: "caller_id_number", header: "Caller", sortable: true },
  { key: "destination_number", header: "Destination", sortable: true },
  { key: "start_stamp", header: "Start", className: "num", sortable: true, cell: (r) => (r.start_stamp ? new Date(r.start_stamp).toLocaleString() : "—") },
  { key: "duration", header: "Duration", className: "num", sortable: true, cell: (r) => `${r.duration ?? 0}s` },
];

export function Cdr() {
  return (
    <>
      <PageHeader title="Call Detail Records" description="History of inbound and outbound calls" />
      <DataTable<CdrRow>
        resource="/cdr"
        columns={columns}
        rowKey={(r) => r.cdr_uuid}
        perPage={20}
        searchPlaceholder="Search CDR…"
        emptyTitle="No CDRs"
        emptyDescription="Call records will appear here once calls are made"
        emptyIcon={Phone}
      />
    </>
  );
}
