import * as React from "react";
import { DataTable } from "../components/DataTable";
import type { Column } from "../components/types";
import { PageHeader } from "../components/shell/PageHeader";

export function Registrations() {
  const columns: Column<any>[] = [
    { key: "reg_user", header: "User", className: "num", cell: (r: any) => r.reg_user ?? r.user ?? r.sip_user ?? "—" },
    { key: "realm", header: "Realm", cell: (r: any) => r.realm ?? r.domain ?? "—" },
    { key: "network_ip", header: "Contact / IP", className: "font-mono text-xs", cell: (r: any) => <span className="font-mono text-xs break-all">{r.network_ip ?? r.contact ?? "—"}</span> },
    { key: "status", header: "Status", cell: (r: any) => r.status ?? "Registered" },
    { key: "expires", header: "Expires", cell: (r: any) => r.expires ?? "—" },
  ];
  return (
    <>
      <PageHeader title="Registrations" description="SIP registrations and contacts" />
      <DataTable resource="/registrations" columns={columns} perPage={20} searchPlaceholder="Search registrations…" emptyTitle="No registrations" />
    </>
  );
}
