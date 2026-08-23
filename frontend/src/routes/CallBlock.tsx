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
import { Ban } from "lucide-react";

export function CallBlock() {
  const [reloadKey, setReloadKey] = React.useState(0);
  const reload = () => setReloadKey((k) => k + 1);
  const [open, setOpen] = React.useState(false);
  const [json, setJson] = React.useState(`{"call_block_number": "8005551212"}`);
  const columns: Column<any>[] = [
    { key: "call_block_number", header: "Number", className: "font-mono text-xs", cell: (r: any) => r["call_block_number"] ?? r.name ?? JSON.stringify(r).slice(0, 60) },
    { key: "details", header: "Details", cell: (r: any) => <span className="text-muted-foreground text-xs break-all">{JSON.stringify(r).slice(0, 80)}</span> },
  ];
  const createFn = async () => {
    try {
      const body = JSON.parse(json);
      await apiFetch("/call_block", { method: "POST", body: JSON.stringify(body) });
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
        <div><Label>JSON payload *</Label><textarea value={json} onChange={(e) => setJson(e.target.value)} rows={6} className="w-full rounded border p-2 font-mono text-xs" /></div>
        <div className="text-xs text-muted-foreground">Required field shown above — edit JSON and press Create.</div>
      </FormDialog>
    </>
  );
}
