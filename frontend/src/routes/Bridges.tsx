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

export function Bridges() {
  const [open, setOpen] = React.useState(false);
  const [form,setForm]=React.useState({ bridge_name: "test-bridge", description: "" });
  const [reloadKey, setReloadKey] = React.useState(0);
  const reload = () => setReloadKey((k) => k + 1);

  const { run: create, pending } = useAsyncAction(async () => {
    try {
      await apiFetch("/bridges", { method: "POST", body: JSON.stringify(form) });
      toast("Bridge created", { variant: "success" });
      setOpen(false);
      reload();
    } catch (err) {
      toastError(err);
    }
  });

  const columns: Column<any>[] = [
    { key: "bridge_name", header: "Name", className: "num", cell: (r: any) => r["bridge_name"] ?? r.name ?? JSON.stringify(r).slice(0,60) },
    { key: "description", header: "Description", cell: (r:any) => (r as any).description || (r as any)["description"] || "—" }
  ];

  return (
    <>
      <PageHeader title="Bridges" description="SIP bridges and outbound routing paths" actions={<Button onClick={() => setOpen(true)}>Add Bridges</Button>} />
      <DataTable resource="/bridges" columns={columns} perPage={20} reloadKey={reloadKey} searchPlaceholder="Search bridges…" emptyTitle="No bridges" emptyDescription="Create via Add button" />
      <FormDialog open={open} onOpenChange={setOpen} title="Add Bridges" onSubmit={create} pending={pending} submitLabel="Create" pendingLabel="Creating…">
        <div><Label>Name *</Label><Input value={form.bridge_name} onChange={e=>setForm({...form,bridge_name:e.target.value})} placeholder="test-bridge" /></div>
        <div><Label>Description</Label><Input value={form.description} onChange={e=>setForm({...form,description:e.target.value})} /></div>
      </FormDialog>
    </>
  );
}
