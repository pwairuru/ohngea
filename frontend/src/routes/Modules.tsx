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

export function Modules() {
  const [open, setOpen] = React.useState(false);
  const [json, setJson] = React.useState(`{
  "module_label": "Test",
  "module_name": "test_mod"
}`);
  const [reloadKey, setReloadKey] = React.useState(0);
  const reload = () => setReloadKey((k) => k + 1);

  const { run: create, pending } = useAsyncAction(async () => {
    try {
      await apiFetch("/modules", { method: "POST", body: JSON.stringify(JSON.parse(json)) });
      toast("Module created", { variant: "success" });
      setOpen(false);
      reload();
    } catch (err) {
      toastError(err);
    }
  });

  const columns: Column<any>[] = [
    { key: "module_name", header: "Name", className: "num", cell: (r: any) => r["module_name"] ?? r["module_name"] ?? r.name ?? JSON.stringify(r).slice(0,60) },
    { key: "details", header: "Details", cell: (r: any) => <span className="text-muted-foreground text-xs break-all">{JSON.stringify(r).slice(0, 80)}</span> },
  ];

  return (
    <>
      <PageHeader title="Modules" description="Loadable FreeSWITCH modules and status" actions={<Button onClick={() => setOpen(true)}>Add Modules</Button>} />
      <DataTable resource="/modules" columns={columns} perPage={20} reloadKey={reloadKey} searchPlaceholder="Search modules…" emptyTitle="No modules" emptyDescription="Create via Add button" />
      <FormDialog open={open} onOpenChange={setOpen} title="Add Modules" onSubmit={create} pending={pending} submitLabel="Create" pendingLabel="Creating…">
        <div><Label>JSON payload *</Label><textarea value={json} onChange={(e) => setJson(e.target.value)} rows={6} className="w-full rounded border p-2 font-mono text-xs" /></div>
          <div className="text-xs text-muted-foreground">Required field shown above — edit JSON and press Create.</div>
      </FormDialog>
    </>
  );
}
