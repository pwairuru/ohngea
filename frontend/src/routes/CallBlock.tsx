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
import { Ban } from "lucide-react";

export function CallBlock() {
  const [reloadKey, setReloadKey] = React.useState(0);
  const reload = () => setReloadKey((k) => k + 1);
  const [open, setOpen] = React.useState(false);
  const [form,setForm]=React.useState({ call_block_number: "8005551212", description: "" });
  const columns: Column<any>[] = [
    { key: "call_block_number", header: "Number", className: "num", cell: (r: any) => r["call_block_number"] ?? r.name ?? JSON.stringify(r).slice(0, 60) },
    { key: "description", header: "Description", cell: (r:any) => (r as any).description || (r as any)["description"] || "—" }
  ];
  const createFn = async () => {
    try {
      await apiFetch("/call_block", { method: "POST", body: JSON.stringify(form) });
      toast("Call block created", { variant: "success" });
      setOpen(false);
      reload();
    } catch (err) {
      toastError(err);
    }
  };
  const { run: create, pending } = useAsyncAction(createFn);
  return (
    <>
      <PageHeader title="Call Block" description="Blocked numbers" actions={<Button onClick={() => setOpen(true)}>Add Call Block</Button>} />
      <DataTable
        resource="/call_block"
        columns={columns}
        reloadKey={reloadKey}
        perPage={20}
        searchPlaceholder="Search call block…"
        emptyTitle="No call block"
        emptyDescription="Create your first call block entry"
        emptyIcon={Ban}
        emptyAction={<Button onClick={() => setOpen(true)}><Ban className="h-4 w-4" /> Add Call Block</Button>}
      />
      <FormDialog open={open} onOpenChange={setOpen} title="Add Call Block" onSubmit={create} pending={pending} submitLabel="Create" pendingLabel="Creating…">
        <div><Label>Number *</Label><Input value={form.call_block_number} onChange={e=>setForm({...form,call_block_number:e.target.value})} placeholder="8005551212" /></div>
        <div><Label>Description</Label><Input value={form.description} onChange={e=>setForm({...form,description:e.target.value})} /></div>
      </FormDialog>
    </>
  );
}
