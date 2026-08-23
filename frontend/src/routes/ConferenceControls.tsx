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

export function ConferenceControls() {
  const [open, setOpen] = React.useState(false);
  const [form,setForm]=React.useState({ control_name: "conf-ctrl", description: "" });
  const [reloadKey, setReloadKey] = React.useState(0);
  const reload = () => setReloadKey((k) => k + 1);

  const { run: create, pending } = useAsyncAction(async () => {
    try {
      await apiFetch("/conference_controls", { method: "POST", body: JSON.stringify(form) });
      toast("Conference control created", { variant: "success" });
      setOpen(false);
      reload();
    } catch (err) {
      toastError(err);
    }
  });

  const columns: Column<any>[] = [
    { key: "control_name", header: "Name", className: "num", cell: (r: any) => r["control_name"] ?? r.name ?? JSON.stringify(r).slice(0,60) },
    { key: "description", header: "Description", cell: (r:any) => (r as any).description || (r as any)["description"] || "—" }
  ];

  return (
    <>
      <PageHeader title="Conference Controls" description="Conference control key mappings" actions={<Button onClick={() => setOpen(true)}>Add Conference Controls</Button>} />
      <DataTable resource="/conference_controls" columns={columns} perPage={20} reloadKey={reloadKey} searchPlaceholder="Search conference controls…" emptyTitle="No conference controls" emptyDescription="Create via Add button" />
      <FormDialog open={open} onOpenChange={setOpen} title="Add Conference Controls" onSubmit={create} pending={pending} submitLabel="Create" pendingLabel="Creating…">
        <div><Label>Control *</Label><Input value={form.control_name} onChange={e=>setForm({...form,control_name:e.target.value})} placeholder="conf-ctrl" /></div>
        <div><Label>Description</Label><Input value={form.description} onChange={e=>setForm({...form,description:e.target.value})} /></div>
      </FormDialog>
    </>
  );
}
