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
import { Clock } from "lucide-react";

export function TimeConditions() {
  const [reloadKey, setReloadKey] = React.useState(0);
  const reload = () => setReloadKey((k) => k + 1);
  const [open, setOpen] = React.useState(false);
  const [form,setForm]=React.useState({ time_condition_name: "tc1", description: "" });
  const columns: Column<any>[] = [
    { key: "time_condition_name", header: "Name", className: "num", cell: (r: any) => r["time_condition_name"] ?? r.name ?? JSON.stringify(r).slice(0, 60) },
    { key: "description", header: "Description", cell: (r:any) => (r as any).description || (r as any)["description"] || "—" }
  ];
  const createFn = async () => {
    try {
      await apiFetch("/time_conditions", { method: "POST", body: JSON.stringify(form) });
      toast("Time condition created", { variant: "success" });
      setOpen(false);
      reload();
    } catch (err) {
      toastError(err);
    }
  };
  const { run: create, pending } = useAsyncAction(createFn);
  return (
    <>
      <PageHeader title="Time Conditions" description="Time-based routing rules" actions={<Button onClick={() => setOpen(true)}>Add Time Conditions</Button>} />
      <DataTable
        resource="/time_conditions"
        columns={columns}
        reloadKey={reloadKey}
        perPage={20}
        searchPlaceholder="Search time conditions…"
        emptyTitle="No time conditions"
        emptyDescription="Create your first time conditions entry"
        emptyIcon={Clock}
        emptyAction={<Button onClick={() => setOpen(true)}><Clock className="h-4 w-4" /> Add Time Conditions</Button>}
      />
      <FormDialog open={open} onOpenChange={setOpen} title="Add Time Conditions" onSubmit={create} pending={pending} submitLabel="Create" pendingLabel="Creating…">
        <div><Label>Name *</Label><Input value={form.time_condition_name} onChange={e=>setForm({...form,time_condition_name:e.target.value})} placeholder="tc1" /></div>
        <div><Label>Description</Label><Input value={form.description} onChange={e=>setForm({...form,description:e.target.value})} /></div>
      </FormDialog>
    </>
  );
}
