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

export function Modules() {
  const [open, setOpen] = React.useState(false);
  const [form,setForm]=React.useState({ module_label: "Test", module_name: "test_mod" });
  const [reloadKey, setReloadKey] = React.useState(0);
  const reload = () => setReloadKey((k) => k + 1);

  const { run: create, pending } = useAsyncAction(async () => {
    try {
      await apiFetch("/modules", { method: "POST", body: JSON.stringify(form) });
      toast("Module created", { variant: "success" });
      setOpen(false);
      reload();
    } catch (err) {
      toastError(err);
    }
  });

  const columns: Column<any>[] = [
    { key: "module_name", header: "Name", className: "num", cell: (r: any) => r["module_name"] ?? r["module_name"] ?? r.name ?? JSON.stringify(r).slice(0,60) },
    { key: "module_label", header: "Label", cell: (r:any) => (r as any).module_label || (r as any)["module_label"] || "—" }
  ];

  return (
    <>
      <PageHeader title="Modules" description="Loadable FreeSWITCH modules and status" actions={<Button onClick={() => setOpen(true)}>Add Modules</Button>} />
      <DataTable resource="/modules" columns={columns} perPage={20} reloadKey={reloadKey} searchPlaceholder="Search modules…" emptyTitle="No modules" emptyDescription="Create via Add button" />
      <FormDialog open={open} onOpenChange={setOpen} title="Add Modules" onSubmit={create} pending={pending} submitLabel="Create" pendingLabel="Creating…">
        <div><Label>Label *</Label><Input value={form.module_label} onChange={e=>setForm({...form,module_label:e.target.value})} placeholder="Test" /></div>
        <div><Label>Name</Label><Input value={form.module_name} onChange={e=>setForm({...form,module_name:e.target.value})} placeholder="test_mod" /></div>
      </FormDialog>
    </>
  );
}
