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

export function Fifo() {
  const [open, setOpen] = React.useState(false);
  const [form,setForm]=React.useState({ fifo_name: "test-fifo", fifo_extension: "5001" });
  const [reloadKey, setReloadKey] = React.useState(0);
  const reload = () => setReloadKey((k) => k + 1);

  const { run: create, pending } = useAsyncAction(async () => {
    try {
      await apiFetch("/fifo", { method: "POST", body: JSON.stringify(form) });
      toast("FIFO created", { variant: "success" });
      setOpen(false);
      reload();
    } catch (err) {
      toastError(err);
    }
  });

  const columns: Column<any>[] = [
    { key: "fifo_name", header: "Name", className: "num", cell: (r: any) => r["fifo_name"] ?? r["fifo_extension"] ?? r.name ?? JSON.stringify(r).slice(0,60) },
    { key: "fifo_extension", header: "Extension", cell: (r:any) => (r as any).fifo_extension || (r as any)["fifo_extension"] || "—" }
  ];

  return (
    <>
      <PageHeader title="Fifo" description="FIFO queues and overflow handling" actions={<Button onClick={() => setOpen(true)}>Add Fifo</Button>} />
      <DataTable resource="/fifo" columns={columns} perPage={20} reloadKey={reloadKey} searchPlaceholder="Search fifo…" emptyTitle="No fifo" emptyDescription="Create via Add button" />
      <FormDialog open={open} onOpenChange={setOpen} title="Add Fifo" onSubmit={create} pending={pending} submitLabel="Create" pendingLabel="Creating…">
        <div><Label>Name *</Label><Input value={form.fifo_name} onChange={e=>setForm({...form,fifo_name:e.target.value})} placeholder="test-fifo" /></div>
        <div><Label>Extension</Label><Input value={form.fifo_extension} onChange={e=>setForm({...form,fifo_extension:e.target.value})} placeholder="5001" /></div>
      </FormDialog>
    </>
  );
}
