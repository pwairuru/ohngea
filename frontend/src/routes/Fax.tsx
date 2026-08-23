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

export function Fax() {
  const [open, setOpen] = React.useState(false);
  const [form,setForm]=React.useState({ fax_name: "fax1", fax_extension: "4001" });
  const [reloadKey, setReloadKey] = React.useState(0);
  const reload = () => setReloadKey((k) => k + 1);

  const { run: create, pending } = useAsyncAction(async () => {
    try {
      await apiFetch("/fax", { method: "POST", body: JSON.stringify(form) });
      toast("Fax created", { variant: "success" });
      setOpen(false);
      reload();
    } catch (err) {
      toastError(err);
    }
  });

  const columns: Column<any>[] = [
    { key: "fax_name", header: "Name", className: "num", cell: (r: any) => r["fax_name"] ?? r["fax_extension"] ?? r.name ?? JSON.stringify(r).slice(0,60) },
    { key: "fax_extension", header: "Extension", cell: (r:any) => (r as any).fax_extension || (r as any)["fax_extension"] || "—" }
  ];

  return (
    <>
      <PageHeader title="Fax" description="Fax extensions and delivery settings" actions={<Button onClick={() => setOpen(true)}>Add Fax</Button>} />
      <DataTable resource="/fax" columns={columns} perPage={20} reloadKey={reloadKey} searchPlaceholder="Search fax…" emptyTitle="No fax" emptyDescription="Create via Add button" />
      <FormDialog open={open} onOpenChange={setOpen} title="Add Fax" onSubmit={create} pending={pending} submitLabel="Create" pendingLabel="Creating…">
        <div><Label>Fax Name *</Label><Input value={form.fax_name} onChange={e=>setForm({...form,fax_name:e.target.value})} placeholder="fax1" /></div>
        <div><Label>Extension</Label><Input value={form.fax_extension} onChange={e=>setForm({...form,fax_extension:e.target.value})} placeholder="4001" /></div>
      </FormDialog>
    </>
  );
}
