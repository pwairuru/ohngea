import * as React from "react";
import { apiFetch } from "../lib/api";
import { DataTable } from "../components/DataTable";
import type { Column } from "../components/types";
import { PageHeader } from "../components/shell/PageHeader";
import { Button } from "../components/ui/button";
import { Label } from "../components/ui/label";
import { FormDialog } from "../components/FormDialog";
import { toast, toastError } from "../lib/toast";
import { useAsyncAction } from "../lib/useAsyncAction";
import { Users } from "lucide-react";

export function RingGroups() {
  const [reloadKey, setReloadKey] = React.useState(0);
  const reload = () => setReloadKey((k) => k + 1);
  const [open, setOpen] = React.useState(false);
  const [json, setJson] = React.useState(`{"ring_group_name": "rg1",
  "ring_group_extension": "2001"}`);
  const columns: Column<any>[] = [
    { key: "ring_group_name", header: "Name", className: "num", cell: (r: any) => r["ring_group_name"] ?? r.name ?? JSON.stringify(r).slice(0, 60) },
    { key: "details", header: "Details", cell: (r: any) => <span className="text-muted-foreground text-xs break-all">{JSON.stringify(r).slice(0, 80)}</span> },
  ];
  const createFn = async () => {
    try {
      const body = JSON.parse(json);
      await apiFetch("/ring_groups", { method: "POST", body: JSON.stringify(body) });
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
        <div><Label>JSON payload *</Label><textarea value={json} onChange={(e) => setJson(e.target.value)} rows={6} className="w-full rounded border p-2 font-mono text-xs" /></div>
        <div className="text-xs text-muted-foreground">Required field shown above — edit JSON and press Create.</div>
      </FormDialog>
    </>
  );
}
