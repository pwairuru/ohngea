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

export function ConferenceCenters() {
  const [open, setOpen] = React.useState(false);
  const [form,setForm]=React.useState({ conference_center_name: "conf-center", conference_center_extension: "7001" });
  const [reloadKey, setReloadKey] = React.useState(0);
  const reload = () => setReloadKey((k) => k + 1);

  const { run: create, pending } = useAsyncAction(async () => {
    try {
      await apiFetch("/conference_centers", { method: "POST", body: JSON.stringify(form) });
      toast("Conference center created", { variant: "success" });
      setOpen(false);
      reload();
    } catch (err) {
      toastError(err);
    }
  });

  const columns: Column<any>[] = [
    { key: "conference_center_name", header: "Name", className: "num", cell: (r: any) => r["conference_center_name"] ?? r["conference_center_extension"] ?? r.name ?? JSON.stringify(r).slice(0,60) },
    { key: "conference_center_extension", header: "Extension", cell: (r:any) => (r as any).conference_center_extension || (r as any)["conference_center_extension"] || "—" }
  ];

  return (
    <>
      <PageHeader title="Conference Centers" description="Conference center instances and rooms" actions={<Button onClick={() => setOpen(true)}>Add Conference Centers</Button>} />
      <DataTable resource="/conference_centers" columns={columns} perPage={20} reloadKey={reloadKey} searchPlaceholder="Search conference centers…" emptyTitle="No conference centers" emptyDescription="Create via Add button" />
      <FormDialog open={open} onOpenChange={setOpen} title="Add Conference Centers" onSubmit={create} pending={pending} submitLabel="Create" pendingLabel="Creating…">
        <div><Label>Name *</Label><Input value={form.conference_center_name} onChange={e=>setForm({...form,conference_center_name:e.target.value})} placeholder="conf-center" /></div>
        <div><Label>Extension</Label><Input value={form.conference_center_extension} onChange={e=>setForm({...form,conference_center_extension:e.target.value})} placeholder="7001" /></div>
      </FormDialog>
    </>
  );
}
