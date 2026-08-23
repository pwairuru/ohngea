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

export function VoicemailGreetings() {
  const [open, setOpen] = React.useState(false);
  const [json, setJson] = React.useState(`{
  "greeting_name": "greet1"
}`);
  const [reloadKey, setReloadKey] = React.useState(0);
  const reload = () => setReloadKey((k) => k + 1);

  const { run: create, pending } = useAsyncAction(async () => {
    try {
      await apiFetch("/voicemail_greetings", { method: "POST", body: JSON.stringify(JSON.parse(json)) });
      toast("Greeting created", { variant: "success" });
      setOpen(false);
      reload();
    } catch (err) {
      toastError(err);
    }
  });

  const columns: Column<any>[] = [
    { key: "greeting_name", header: "Name", className: "num", cell: (r: any) => r["greeting_name"] ?? r.name ?? JSON.stringify(r).slice(0,60) },
    { key: "details", header: "Details", cell: (r: any) => <span className="text-muted-foreground text-xs break-all">{JSON.stringify(r).slice(0, 80)}</span> },
  ];

  return (
    <>
      <PageHeader title="Voicemail Greetings" description="Voicemail greeting audio files" actions={<Button onClick={() => setOpen(true)}>Add Voicemail Greetings</Button>} />
      <DataTable resource="/voicemail_greetings" columns={columns} perPage={20} reloadKey={reloadKey} searchPlaceholder="Search voicemail greetings…" emptyTitle="No voicemail greetings" emptyDescription="Create via Add button" />
      <Dialog open={open} onOpenChange={setOpen}>
        <DialogHeader><DialogTitle>Add Voicemail Greetings</DialogTitle></DialogHeader>
        <div className="space-y-3">
          <div><Label>JSON payload *</Label><textarea value={json} onChange={(e) => setJson(e.target.value)} rows={6} className="w-full rounded border p-2 font-mono text-xs" /></div>
          <div className="text-xs text-muted-foreground">Required field shown above — edit JSON and press Create.</div>
          <Button onClick={create} disabled={pending} className="w-full">{pending ? "Creating…" : "Create"}</Button>
        </div>
      </Dialog>
    </>
  );
}
