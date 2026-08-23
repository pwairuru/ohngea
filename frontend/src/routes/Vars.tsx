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

export function Vars() {
  const [open, setOpen] = React.useState(false);
  const [form,setForm]=React.useState({ var_name: "test-var", var_value: "val", var_category: "testing" });
  const [reloadKey, setReloadKey] = React.useState(0);
  const reload = () => setReloadKey((k) => k + 1);

  const { run: create, pending } = useAsyncAction(async () => {
    try {
      await apiFetch("/vars", { method: "POST", body: JSON.stringify(form) });
      toast("Var created", { variant: "success" });
      setOpen(false);
      reload();
    } catch (err) {
      toastError(err);
    }
  });

  const columns: Column<any>[] = [
    { key: "var_name", header: "Name", className: "num", cell: (r: any) => r["var_name"] ?? r["var_value"] ?? r.name ?? JSON.stringify(r).slice(0,60) },
    { key: "var_value", header: "Value", cell: (r:any) => (r as any).var_value || (r as any)["var_value"] || "—" },
    { key: "var_category", header: "Category", cell: (r:any) => (r as any).var_category || (r as any)["var_category"] || "—" }
  ];

  return (
    <>
      <PageHeader title="Vars" description="Channel variables and global vars" actions={<Button onClick={() => setOpen(true)}>Add Vars</Button>} />
      <DataTable resource="/vars" columns={columns} perPage={20} reloadKey={reloadKey} searchPlaceholder="Search vars…" emptyTitle="No vars" emptyDescription="Create via Add button" />
      <FormDialog open={open} onOpenChange={setOpen} title="Add Vars" onSubmit={create} pending={pending} submitLabel="Create" pendingLabel="Creating…">
        <div><Label>Name *</Label><Input value={form.var_name} onChange={e=>setForm({...form,var_name:e.target.value})} placeholder="test-var" /></div>
        <div><Label>Value</Label><Input value={form.var_value} onChange={e=>setForm({...form,var_value:e.target.value})} placeholder="val" /></div>
        <div><Label>Category</Label><Input value={form.var_category} onChange={e=>setForm({...form,var_category:e.target.value})} placeholder="testing" /></div>
      </FormDialog>
    </>
  );
}
