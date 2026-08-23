import * as React from "react";
import { apiFetch } from "../lib/api";
import { DataTable } from "../components/DataTable";
import type { Column } from "../components/types";
import { PageHeader } from "../components/shell/PageHeader";
import { Button } from "../components/ui/button";
import { Label } from "../components/ui/label";
import { Dialog, DialogHeader, DialogTitle } from "../components/ui/dialog";
import { toast, toastError } from "../lib/toast";
import { useAsyncAction } from "../lib/useAsyncAction";

export function EmailQueue() {
  const [open, setOpen] = React.useState(false);
  const [json, setJson] = React.useState(`{
  "email_from": "a@test.com",
  "email_to": "b@test.com",
  "email_subject": "test"
}`);
  const [reloadKey, setReloadKey] = React.useState(0);
  const reload = () => setReloadKey((k) => k + 1);

  const { run: create, pending } = useAsyncAction(async () => {
    try {
      await apiFetch("/email_queue", { method: "POST", body: JSON.stringify(JSON.parse(json)) });
      toast("Email created", { variant: "success" });
      setOpen(false);
      reload();
    } catch (err) {
      toastError(err);
    }
  });

  const columns: Column<any>[] = [
    { key: "email_subject", header: "Name", className: "num", cell: (r: any) => r["email_subject"] ?? r["email_to"] ?? r.name ?? JSON.stringify(r).slice(0,60) },
    { key: "details", header: "Details", cell: (r: any) => <span className="text-muted-foreground text-xs break-all">{JSON.stringify(r).slice(0, 80)}</span> },
  ];

  return (
    <>
      <PageHeader title="Email Queue" description="Outbound email queue and delivery" actions={<Button onClick={() => setOpen(true)}>Add Email Queue</Button>} />
      <DataTable resource="/email_queue" columns={columns} perPage={20} reloadKey={reloadKey} searchPlaceholder="Search email queue…" emptyTitle="No email queue" emptyDescription="Create via Add button" />
      <Dialog open={open} onOpenChange={setOpen}>
        <DialogHeader><DialogTitle>Add Email Queue</DialogTitle></DialogHeader>
        <div className="space-y-3">
          <div><Label>JSON payload *</Label><textarea value={json} onChange={(e) => setJson(e.target.value)} rows={6} className="w-full rounded border p-2 font-mono text-xs" /></div>
          <div className="text-xs text-muted-foreground">Required field shown above — edit JSON and press Create.</div>
          <Button onClick={create} disabled={pending} className="w-full">{pending ? "Creating…" : "Create"}</Button>
        </div>
      </Dialog>
    </>
  );
}
