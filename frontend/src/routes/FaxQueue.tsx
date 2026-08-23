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

export function FaxQueue() {
  const [open, setOpen] = React.useState(false);
  const [form,setForm]=React.useState({ fax_number: "5551234", fax_status: "waiting" });
  const [reloadKey, setReloadKey] = React.useState(0);
  const reload = () => setReloadKey((k) => k + 1);

  const { run: create, pending } = useAsyncAction(async () => {
    try {
      await apiFetch("/fax_queue", { method: "POST", body: JSON.stringify(form) });
      toast("Fax queue created", { variant: "success" });
      setOpen(false);
      reload();
    } catch (err) {
      toastError(err);
    }
  });

  const columns: Column<any>[] = [
    { key: "fax_number", header: "Name", className: "num", cell: (r: any) => r["fax_number"] ?? r["fax_status"] ?? r.name ?? JSON.stringify(r).slice(0,60) },
    { key: "fax_status", header: "Status", cell: (r:any) => (r as any).fax_status || (r as any)["fax_status"] || "—" }
  ];

  return (
    <>
      <PageHeader title="Fax Queue" description="Outbound fax queue and status" actions={<Button onClick={() => setOpen(true)}>Add Fax Queue</Button>} />
      <DataTable resource="/fax_queue" columns={columns} perPage={20} reloadKey={reloadKey} searchPlaceholder="Search fax queue…" emptyTitle="No fax queue" emptyDescription="Create via Add button" />
      <FormDialog open={open} onOpenChange={setOpen} title="Add Fax Queue" onSubmit={create} pending={pending} submitLabel="Create" pendingLabel="Creating…">
        <div><Label>Number *</Label><Input value={form.fax_number} onChange={e=>setForm({...form,fax_number:e.target.value})} placeholder="5551234" /></div>
        <div><Label>Status</Label><Input value={form.fax_status} onChange={e=>setForm({...form,fax_status:e.target.value})} placeholder="waiting" /></div>
      </FormDialog>
    </>
  );
}
