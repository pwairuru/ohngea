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

export function VoicemailGreetings() {
  const [open, setOpen] = React.useState(false);
  const [form,setForm]=React.useState({ greeting_name: "greet1", description: "" });
  const [reloadKey, setReloadKey] = React.useState(0);
  const reload = () => setReloadKey((k) => k + 1);

  const { run: create, pending } = useAsyncAction(async () => {
    try {
      await apiFetch("/voicemail_greetings", { method: "POST", body: JSON.stringify(form) });
      toast("Greeting created", { variant: "success" });
      setOpen(false);
      reload();
    } catch (err) {
      toastError(err);
    }
  });

  const columns: Column<any>[] = [
    { key: "greeting_name", header: "Name", className: "num", cell: (r: any) => r["greeting_name"] ?? r.name ?? JSON.stringify(r).slice(0,60) },
    { key: "description", header: "Description", cell: (r:any) => (r as any).description || (r as any)["description"] || "—" }
  ];

  return (
    <>
      <PageHeader title="Voicemail Greetings" description="Voicemail greeting audio files" actions={<Button onClick={() => setOpen(true)}>Add Voicemail Greetings</Button>} />
      <DataTable resource="/voicemail_greetings" columns={columns} perPage={20} reloadKey={reloadKey} searchPlaceholder="Search voicemail greetings…" emptyTitle="No voicemail greetings" emptyDescription="Create via Add button" />
      <FormDialog open={open} onOpenChange={setOpen} title="Add Voicemail Greetings" onSubmit={create} pending={pending} submitLabel="Create" pendingLabel="Creating…">
        <div><Label>Name *</Label><Input value={form.greeting_name} onChange={e=>setForm({...form,greeting_name:e.target.value})} placeholder="greet1" /></div>
        <div><Label>Description</Label><Input value={form.description} onChange={e=>setForm({...form,description:e.target.value})} /></div>
      </FormDialog>
    </>
  );
}
