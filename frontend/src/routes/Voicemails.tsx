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

export function Voicemails() {
  const [open, setOpen] = React.useState(false);
  const [json, setJson] = React.useState(`{
  "voicemail_id": "1001",
  "voicemail_password": "1234"
}`);
  const [reloadKey, setReloadKey] = React.useState(0);
  const reload = () => setReloadKey((k) => k + 1);

  const { run: create, pending } = useAsyncAction(async () => {
    try {
      await apiFetch("/voicemails", { method: "POST", body: JSON.stringify(JSON.parse(json)) });
      toast("Voicemail created", { variant: "success" });
      setOpen(false);
      reload();
    } catch (err) {
      toastError(err);
    }
  });

  const columns: Column<any>[] = [
    { key: "voicemail_id", header: "Name", className: "num", cell: (r: any) => r["voicemail_id"] ?? r["voicemail_password"] ?? r.name ?? JSON.stringify(r).slice(0,60) },
    { key: "details", header: "Details", cell: (r: any) => <span className="text-muted-foreground text-xs break-all">{JSON.stringify(r).slice(0, 80)}</span> },
  ];

  return (
    <>
      <PageHeader title="Voicemails" description="Voicemail boxes and mailbox settings" actions={<Button onClick={() => setOpen(true)}>Add Voicemails</Button>} />
      <DataTable resource="/voicemails" columns={columns} perPage={20} reloadKey={reloadKey} searchPlaceholder="Search voicemails…" emptyTitle="No voicemails" emptyDescription="Create via Add button" />
      <FormDialog open={open} onOpenChange={setOpen} title="Add Voicemails" onSubmit={create} pending={pending} submitLabel="Create" pendingLabel="Creating…">
        <div><Label>JSON payload *</Label><textarea value={json} onChange={(e) => setJson(e.target.value)} rows={6} className="w-full rounded border p-2 font-mono text-xs" /></div>
          <div className="text-xs text-muted-foreground">Required field shown above — edit JSON and press Create.</div>
      </FormDialog>
    </>
  );
}
