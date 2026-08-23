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

export function EventGuard() {
  const [open, setOpen] = React.useState(false);
  const [form,setForm]=React.useState({ hostname: "pbx1", ip_address: "1.2.3.4" });
  const [reloadKey, setReloadKey] = React.useState(0);
  const reload = () => setReloadKey((k) => k + 1);

  const { run: create, pending } = useAsyncAction(async () => {
    try {
      await apiFetch("/event_guard", { method: "POST", body: JSON.stringify(form) });
      toast("Event guard created", { variant: "success" });
      setOpen(false);
      reload();
    } catch (err) {
      toastError(err);
    }
  });

  const columns: Column<any>[] = [
    { key: "hostname", header: "Name", className: "num", cell: (r: any) => r["hostname"] ?? r["ip_address"] ?? r.name ?? JSON.stringify(r).slice(0,60) },
    { key: "ip_address", header: "IP Address", cell: (r:any) => (r as any).ip_address || (r as any)["ip_address"] || "—" }
  ];

  return (
    <>
      <PageHeader title="Event Guard" description="Event guard and heartbeat monitors" actions={<Button onClick={() => setOpen(true)}>Add Event Guard</Button>} />
      <DataTable resource="/event_guard" columns={columns} perPage={20} reloadKey={reloadKey} searchPlaceholder="Search event guard…" emptyTitle="No event guard" emptyDescription="Create via Add button" />
      <FormDialog open={open} onOpenChange={setOpen} title="Add Event Guard" onSubmit={create} pending={pending} submitLabel="Create" pendingLabel="Creating…">
        <div><Label>Hostname *</Label><Input value={form.hostname} onChange={e=>setForm({...form,hostname:e.target.value})} placeholder="pbx1" /></div>
        <div><Label>IP Address</Label><Input value={form.ip_address} onChange={e=>setForm({...form,ip_address:e.target.value})} placeholder="1.2.3.4" /></div>
      </FormDialog>
    </>
  );
}
