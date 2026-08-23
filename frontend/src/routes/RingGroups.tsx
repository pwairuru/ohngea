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
import { Users } from "lucide-react";

export function RingGroups() {
  const [reloadKey, setReloadKey] = React.useState(0);
  const reload = () => setReloadKey((k) => k + 1);
  const [open, setOpen] = React.useState(false);
  const [form,setForm]=React.useState({ ring_group_name: "rg1", ring_group_extension: "2001" });
  const columns: Column<any>[] = [
    { key: "ring_group_name", header: "Name", className: "num", cell: (r: any) => r["ring_group_name"] ?? r.name ?? JSON.stringify(r).slice(0, 60) },
    { key: "ring_group_extension", header: "Extension", cell: (r:any) => (r as any).ring_group_extension || (r as any)["ring_group_extension"] || "—" }
  ];
  const createFn = async () => {
    try {
      await apiFetch("/ring_groups", { method: "POST", body: JSON.stringify(form) });
      toast("Ring group created", { variant: "success" });
      setOpen(false);
      reload();
    } catch (err) {
      toastError(err);
    }
  };
  const { run: create, pending } = useAsyncAction(createFn);
  return (
    <>
      <PageHeader title="Ring Groups" description="Hunt groups for incoming calls" actions={<Button onClick={() => setOpen(true)}>Add Ring Groups</Button>} />
      <DataTable
        resource="/ring_groups"
        columns={columns}
        reloadKey={reloadKey}
        perPage={20}
        searchPlaceholder="Search ring groups…"
        emptyTitle="No ring groups"
        emptyDescription="Create your first ring groups entry"
        emptyIcon={Users}
        emptyAction={<Button onClick={() => setOpen(true)}><Users className="h-4 w-4" /> Add Ring Groups</Button>}
      />
      <FormDialog open={open} onOpenChange={setOpen} title="Add Ring Groups" onSubmit={create} pending={pending} submitLabel="Create" pendingLabel="Creating…">
        <div><Label>Name *</Label><Input value={form.ring_group_name} onChange={e=>setForm({...form,ring_group_name:e.target.value})} placeholder="rg1" /></div>
        <div><Label>Extension</Label><Input value={form.ring_group_extension} onChange={e=>setForm({...form,ring_group_extension:e.target.value})} placeholder="2001" /></div>
      </FormDialog>
    </>
  );
}
