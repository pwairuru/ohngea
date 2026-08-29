import * as React from "react";
import { DataTable } from "../components/DataTable";
import type { Column } from "../components/types";
import { PageHeader } from "../components/shell/PageHeader";
import { Button } from "../components/ui/button";
import { Shield } from "lucide-react";

type GroupRow = {
  group_uuid: string;
  name: string;
  group_level: number;
  group_protected: boolean;
  description?: string;
  domain_name?: string;
};

export function Groups() {
  const [reloadKey, setReloadKey] = React.useState(0);

  const columns: Column<GroupRow>[] = [
    { key: "name", header: "Name", className: "font-mono font-medium", sortable: true },
    { key: "group_level", header: "Level", className: "num", sortable: true },
    { key: "description", header: "Description" },
  ];

  return (
    <>
      <PageHeader
        title="Groups"
        description="Permission groups and roles"
        actions={<Button onClick={() => setReloadKey((k) => k + 1)}>Refresh</Button>}
      />
      <DataTable<GroupRow>
        resource="/groups"
        columns={columns}
        rowKey={(r) => r.group_uuid}
        reloadKey={reloadKey}
        perPage={20}
        searchPlaceholder="Search groups…"
        emptyTitle="No groups"
        emptyDescription="Groups define permission sets per domain"
        emptyIcon={Shield}
      />
    </>
  );
}
