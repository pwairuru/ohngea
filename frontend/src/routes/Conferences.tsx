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

export function Conferences() {
  const [open, setOpen] = React.useState(false);
  const [form,setForm]=React.useState({ conference_name: "conf1", conference_extension: "3001" });
  const [reloadKey, setReloadKey] = React.useState(0);
  const reload = () => setReloadKey((k) => k + 1);

  const { run: create, pending } = useAsyncAction(async () => {
    try {
      await apiFetch("/conferences", { method: "POST", body: JSON.stringify(form) });
      toast("Conference created", { variant: "success" });
      setOpen(false);
      reload();
    } catch (err) {
      toastError(err);
    }
  });

  const columns: Column<any>[] = [
    { key: "conference_name", header: "Name", className: "num", cell: (r: any) => r["conference_name"] ?? r["conference_extension"] ?? r.name ?? JSON.stringify(r).slice(0,60) },
    { key: "conference_extension", header: "Extension", cell: (r:any) => (r as any).conference_extension || (r as any)["conference_extension"] || "—" }
  ];

  return (
    <>
      <PageHeader title="Conferences" description="Audio conference rooms and pins" actions={<Button onClick={() => setOpen(true)}>Add Conferences</Button>} />
      <DataTable resource="/conferences" columns={columns} perPage={20} reloadKey={reloadKey} searchPlaceholder="Search conferences…" emptyTitle="No conferences" emptyDescription="Create via Add button" />
      <FormDialog open={open} onOpenChange={setOpen} title="Add Conferences" onSubmit={create} pending={pending} submitLabel="Create" pendingLabel="Creating…">
        <div><Label>Name *</Label><Input value={form.conference_name} onChange={e=>setForm({...form,conference_name:e.target.value})} placeholder="conf1" /></div>
        <div><Label>Extension</Label><Input value={form.conference_extension} onChange={e=>setForm({...form,conference_extension:e.target.value})} placeholder="3001" /></div>
      </FormDialog>
    </>
  );
}
