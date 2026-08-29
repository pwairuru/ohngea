import * as React from "react";
import { DataTable } from "../components/DataTable";
import type { Column } from "../components/types";
import { PageHeader } from "../components/shell/PageHeader";
import { Badge } from "../components/ui/badge";

export function ActiveConferences() {
  const columns: Column<any>[] = [
    { key: "conference_name", header: "Conference", className: "font-mono text-xs", cell: (r: any) => <span className="font-mono text-xs break-all">{r.conference_name ?? r.name ?? "—"}</span> },
    { key: "member_count", header: "Members", cell: (r: any) => <Badge variant="outline">{String(r.member_count ?? r.members ?? 0)}</Badge> },
    { key: "profile", header: "Profile", cell: (r: any) => r.profile ?? r.conference_profile ?? "—" },
  ];
  return (
    <>
      <PageHeader title="Active Conferences" description="Live conferences (ESL conference list, fallback DB)" />
      <DataTable resource="/active_conferences" columns={columns} perPage={20} searchPlaceholder="Search conferences…" emptyTitle="No active conferences" emptyDescription="No live conference sessions. ESL `conference list` empty or FS offline." />
    </>
  );
}
