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
import { KeyRound } from "lucide-react";

export function PinNumbers() {
  const [reloadKey, setReloadKey] = React.useState(0);
  const reload = () => setReloadKey((k) => k + 1);
  const [open, setOpen] = React.useState(false);
  const [json, setJson] = React.useState(`{"accountcode": "1234",
  "pin_number": "1234"}`);
  const columns: Column<any>[] = [
    { key: "pin_number", header: "PIN", className: "font-mono text-xs", cell: (r: any) => r["pin_number"] ?? r.name ?? JSON.stringify(r).slice(0, 60) },
    { key: "details", header: "Details", cell: (r: any) => <span className="text-muted-foreground text-xs break-all">{JSON.stringify(r).slice(0, 80)}</span> },
  ];
  const createFn = async () => {
    try {
      const body = JSON.parse(json);
      await apiFetch("/pin_numbers", { method: "POST", body: JSON.stringify(body) });
      toast("PIN created", { variant: "success" });
      setOpen(false);
      reload();
    } catch (err) {
      toastError(err);
    }
  };
  const { run: create, pending } = useAsyncAction(createFn);
  return (
    <>
      <PageHeader title="Pin Numbers" description="PIN codes for auth" actions={<Button onClick={() => setOpen(true)}>Add Pin Numbers</Button>} />
      <DataTable
        resource="/pin_numbers"
        columns={columns}
        reloadKey={reloadKey}
        perPage={20}
        searchPlaceholder="Search pin numbers…"
        emptyTitle="No pin numbers"
        emptyDescription="Create your first pin numbers entry"
        emptyIcon={KeyRound}
        emptyAction={<Button onClick={() => setOpen(true)}><KeyRound className="h-4 w-4" /> Add Pin Numbers</Button>}
      />
      <FormDialog open={open} onOpenChange={setOpen} title="Add Pin Numbers" onSubmit={create} pending={pending} submitLabel="Create" pendingLabel="Creating…">
        <div><Label>JSON payload *</Label><textarea value={json} onChange={(e) => setJson(e.target.value)} rows={6} className="w-full rounded border p-2 font-mono text-xs" /></div>
        <div className="text-xs text-muted-foreground">Required field shown above — edit JSON and press Create.</div>
      </FormDialog>
    </>
  );
}
