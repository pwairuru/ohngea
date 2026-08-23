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

export function MusicOnHold() {
  const [open, setOpen] = React.useState(false);
  const [form,setForm]=React.useState({ moh_name: "Default", description: "" });
  const [reloadKey, setReloadKey] = React.useState(0);
  const reload = () => setReloadKey((k) => k + 1);

  const { run: create, pending } = useAsyncAction(async () => {
    try {
      await apiFetch("/music_on_hold", { method: "POST", body: JSON.stringify(form) });
      toast("Music on hold created", { variant: "success" });
      setOpen(false);
      reload();
    } catch (err) {
      toastError(err);
    }
  });

  const columns: Column<any>[] = [
    { key: "moh_name", header: "Name", className: "num", cell: (r: any) => r["moh_name"] ?? r.name ?? JSON.stringify(r).slice(0,60) },
    { key: "description", header: "Description", cell: (r:any) => (r as any).description || (r as any)["description"] || "—" }
  ];

  return (
    <>
      <PageHeader title="Music On Hold" description="Music on hold streams and files" actions={<Button onClick={() => setOpen(true)}>Add Music On Hold</Button>} />
      <DataTable resource="/music_on_hold" columns={columns} perPage={20} reloadKey={reloadKey} searchPlaceholder="Search music on hold…" emptyTitle="No music on hold" emptyDescription="Create via Add button" />
      <FormDialog open={open} onOpenChange={setOpen} title="Add Music On Hold" onSubmit={create} pending={pending} submitLabel="Create" pendingLabel="Creating…">
        <div><Label>Name *</Label><Input value={form.moh_name} onChange={e=>setForm({...form,moh_name:e.target.value})} placeholder="Default" /></div>
        <div><Label>Description</Label><Input value={form.description} onChange={e=>setForm({...form,description:e.target.value})} /></div>
      </FormDialog>
    </>
  );
}
