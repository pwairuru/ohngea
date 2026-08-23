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
import { UserCheck } from "lucide-react";

export function FollowMe() {
  const [reloadKey, setReloadKey] = React.useState(0);
  const reload = () => setReloadKey((k) => k + 1);
  const [open, setOpen] = React.useState(false);
  const [json, setJson] = React.useState(`{"follow_me_number": "1001"}`);
  const columns: Column<any>[] = [
    { key: "follow_me_number", header: "Number", className: "font-mono text-xs", cell: (r: any) => r["follow_me_number"] ?? r.name ?? JSON.stringify(r).slice(0, 60) },
    { key: "details", header: "Details", cell: (r: any) => <span className="text-muted-foreground text-xs break-all">{JSON.stringify(r).slice(0, 80)}</span> },
  ];
  const createFn = async () => {
    try {
      const body = JSON.parse(json);
      await apiFetch("/follow_me", { method: "POST", body: JSON.stringify(body) });
      toast("Follow me created", { variant: "success" });
      setOpen(false);
      reload();
    } catch (err) {
      toastError(err);
    }
  };
  const { run: create, pending } = useAsyncAction(createFn);
  return (
    <>
      <PageHeader title="Follow Me" description="Find-me / follow-me entries" actions={<Button onClick={() => setOpen(true)}>Add Follow Me</Button>} />
      <DataTable
        resource="/follow_me"
        columns={columns}
        reloadKey={reloadKey}
        perPage={20}
        searchPlaceholder="Search follow me…"
        emptyTitle="No follow me"
        emptyDescription="Create your first follow me entry"
        emptyIcon={UserCheck}
        emptyAction={<Button onClick={() => setOpen(true)}><UserCheck className="h-4 w-4" /> Add Follow Me</Button>}
      />
      <FormDialog open={open} onOpenChange={setOpen} title="Add Follow Me" onSubmit={create} pending={pending} submitLabel="Create" pendingLabel="Creating…">
        <div><Label>JSON payload *</Label><textarea value={json} onChange={(e) => setJson(e.target.value)} rows={6} className="w-full rounded border p-2 font-mono text-xs" /></div>
        <div className="text-xs text-muted-foreground">Required field shown above — edit JSON and press Create.</div>
      </FormDialog>
    </>
  );
}
