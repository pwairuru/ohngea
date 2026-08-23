import * as React from "react";
import { apiFetch } from "../lib/api";
import { DataTable } from "../components/DataTable";
import type { Column } from "../components/types";
import { PageHeader } from "../components/shell/PageHeader";
import { Button } from "../components/ui/button";
import { Label } from "../components/ui/label";
import { Input } from "../components/ui/input";
import { FormDialog } from "../components/FormDialog";
import { toast, toastError } from "../lib/toast";
import { useAsyncAction } from "../lib/useAsyncAction";

export function AccessControls() {
  const [open, setOpen] = React.useState(false);
  const [form,setForm]=React.useState({ access_control_name: "lan", description: "" });
  const [reloadKey, setReloadKey] = React.useState(0);
  const reload = () => setReloadKey((k) => k + 1);

  const { run: create, pending } = useAsyncAction(async () => {
    try {
      await apiFetch("/access_controls", { method: "POST", body: JSON.stringify(form) });
      toast("Access control created", { variant: "success" });
      setOpen(false);
      reload();
    } catch (err) {
      toastError(err);
    }
  });

  const columns: Column<any>[] = [
    { key: "access_control_name", header: "Name", className: "num", cell: (r: any) => r["access_control_name"] ?? r.name ?? JSON.stringify(r).slice(0,60) },
    { key: "description", header: "Description", cell: (r:any) => (r as any).description || (r as any)["description"] || "—" }
  ];

  return (
    <>
      <PageHeader title="Access Controls" description="Network ACLs and CIDR allow/deny lists" actions={<Button onClick={() => setOpen(true)}>Add Access Controls</Button>} />
      <DataTable resource="/access_controls" columns={columns} perPage={20} reloadKey={reloadKey} searchPlaceholder="Search access controls…" emptyTitle="No access controls" emptyDescription="Create via Add button" />
      <FormDialog open={open} onOpenChange={setOpen} title="Add Access Controls" onSubmit={create} pending={pending} submitLabel="Create" pendingLabel="Creating…">
        <div><Label>Name *</Label><Input value={form.access_control_name} onChange={e=>setForm({...form,access_control_name:e.target.value})} placeholder="lan" /></div>
        <div><Label>Description</Label><Input value={form.description} onChange={e=>setForm({...form,description:e.target.value})} /></div>
      </FormDialog>
    </>
  );
}
