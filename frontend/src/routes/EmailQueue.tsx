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

export function EmailQueue() {
  const [open, setOpen] = React.useState(false);
  const [form,setForm]=React.useState({ email_from: "a@test.com", email_to: "b@test.com", email_subject: "test" });
  const [reloadKey, setReloadKey] = React.useState(0);
  const reload = () => setReloadKey((k) => k + 1);

  const { run: create, pending } = useAsyncAction(async () => {
    try {
      await apiFetch("/email_queue", { method: "POST", body: JSON.stringify(form) });
      toast("Email created", { variant: "success" });
      setOpen(false);
      reload();
    } catch (err) {
      toastError(err);
    }
  });

  const columns: Column<any>[] = [
    { key: "email_subject", header: "Name", className: "num", cell: (r: any) => r["email_subject"] ?? r["email_to"] ?? r.name ?? JSON.stringify(r).slice(0,60) },
    { key: "email_from", header: "From", cell: (r:any) => (r as any).email_from || (r as any)["email_from"] || "—" },
    { key: "email_to", header: "To", cell: (r:any) => (r as any).email_to || (r as any)["email_to"] || "—" }
  ];

  return (
    <>
      <PageHeader title="Email Queue" description="Outbound email queue and delivery" actions={<Button onClick={() => setOpen(true)}>Add Email Queue</Button>} />
      <DataTable resource="/email_queue" columns={columns} perPage={20} reloadKey={reloadKey} searchPlaceholder="Search email queue…" emptyTitle="No email queue" emptyDescription="Create via Add button" />
      <FormDialog open={open} onOpenChange={setOpen} title="Add Email Queue" onSubmit={create} pending={pending} submitLabel="Create" pendingLabel="Creating…">
        <div><Label>From *</Label><Input value={form.email_from} onChange={e=>setForm({...form,email_from:e.target.value})} placeholder="a@test.com" /></div>
        <div><Label>To</Label><Input value={form.email_to} onChange={e=>setForm({...form,email_to:e.target.value})} placeholder="b@test.com" /></div>
        <div><Label>Subject</Label><Input value={form.email_subject} onChange={e=>setForm({...form,email_subject:e.target.value})} placeholder="test" /></div>
      </FormDialog>
    </>
  );
}
