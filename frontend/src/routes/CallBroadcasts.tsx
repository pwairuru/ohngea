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

export function CallBroadcasts() {
  const [open, setOpen] = React.useState(false);
  const [form,setForm]=React.useState({ broadcast_name: "blast1", description: "" });
  const [reloadKey, setReloadKey] = React.useState(0);
  const reload = () => setReloadKey((k) => k + 1);

  const { run: create, pending } = useAsyncAction(async () => {
    try {
      await apiFetch("/call_broadcasts", { method: "POST", body: JSON.stringify(form) });
      toast("Call broadcast created", { variant: "success" });
      setOpen(false);
      reload();
    } catch (err) {
      toastError(err);
    }
  });

  const columns: Column<any>[] = [
    { key: "broadcast_name", header: "Name", className: "num", cell: (r: any) => r["broadcast_name"] ?? r.name ?? JSON.stringify(r).slice(0,60) },
    { key: "description", header: "Description", cell: (r:any) => (r as any).description || (r as any)["description"] || "—" }
  ];

  return (
    <>
      <PageHeader title="Call Broadcasts" description="Bulk call broadcasts and campaigns" actions={<Button onClick={() => setOpen(true)}>Add Call Broadcasts</Button>} />
      <DataTable resource="/call_broadcasts" columns={columns} perPage={20} reloadKey={reloadKey} searchPlaceholder="Search call broadcasts…" emptyTitle="No call broadcasts" emptyDescription="Create via Add button" />
      <FormDialog open={open} onOpenChange={setOpen} title="Add Call Broadcasts" onSubmit={create} pending={pending} submitLabel="Create" pendingLabel="Creating…">
        <div><Label>Name *</Label><Input value={form.broadcast_name} onChange={e=>setForm({...form,broadcast_name:e.target.value})} placeholder="blast1" /></div>
        <div><Label>Description</Label><Input value={form.description} onChange={e=>setForm({...form,description:e.target.value})} /></div>
      </FormDialog>
    </>
  );
}
