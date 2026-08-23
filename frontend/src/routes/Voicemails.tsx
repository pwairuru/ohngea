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

export function Voicemails() {
  const [open, setOpen] = React.useState(false);
  const [form,setForm]=React.useState({ voicemail_id: "1001", voicemail_password: "1234" });
  const [reloadKey, setReloadKey] = React.useState(0);
  const reload = () => setReloadKey((k) => k + 1);

  const { run: create, pending } = useAsyncAction(async () => {
    try {
      await apiFetch("/voicemails", { method: "POST", body: JSON.stringify(form) });
      toast("Voicemail created", { variant: "success" });
      setOpen(false);
      reload();
    } catch (err) {
      toastError(err);
    }
  });

  const columns: Column<any>[] = [
    { key: "voicemail_id", header: "Name", className: "num", cell: (r: any) => r["voicemail_id"] ?? r["voicemail_password"] ?? r.name ?? JSON.stringify(r).slice(0,60) },
    { key: "voicemail_password", header: "Password", cell: (r:any) => (r as any).voicemail_password || (r as any)["voicemail_password"] || "—" }
  ];

  return (
    <>
      <PageHeader title="Voicemails" description="Voicemail boxes and mailbox settings" actions={<Button onClick={() => setOpen(true)}>Add Voicemails</Button>} />
      <DataTable resource="/voicemails" columns={columns} perPage={20} reloadKey={reloadKey} searchPlaceholder="Search voicemails…" emptyTitle="No voicemails" emptyDescription="Create via Add button" />
      <FormDialog open={open} onOpenChange={setOpen} title="Add Voicemails" onSubmit={create} pending={pending} submitLabel="Create" pendingLabel="Creating…">
        <div><Label>Mailbox *</Label><Input value={form.voicemail_id} onChange={e=>setForm({...form,voicemail_id:e.target.value})} placeholder="1001" /></div>
        <div><Label>Password</Label><Input value={form.voicemail_password} onChange={e=>setForm({...form,voicemail_password:e.target.value})} placeholder="1234" /></div>
      </FormDialog>
    </>
  );
}
