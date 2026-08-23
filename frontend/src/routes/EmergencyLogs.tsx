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

export function EmergencyLogs() {
  const [open, setOpen] = React.useState(false);
  const [form,setForm]=React.useState({ extension: "1001", event: "emergency" });
  const [reloadKey, setReloadKey] = React.useState(0);
  const reload = () => setReloadKey((k) => k + 1);

  const { run: create, pending } = useAsyncAction(async () => {
    try {
      await apiFetch("/emergency_logs", { method: "POST", body: JSON.stringify(form) });
      toast("Emergency log created", { variant: "success" });
      setOpen(false);
      reload();
    } catch (err) {
      toastError(err);
    }
  });

  const columns: Column<any>[] = [
    { key: "extension", header: "Name", className: "num", cell: (r: any) => r["extension"] ?? r["event"] ?? r.name ?? JSON.stringify(r).slice(0,60) },
    { key: "event", header: "Event", cell: (r:any) => (r as any).event || (r as any)["event"] || "—" }
  ];

  return (
    <>
      <PageHeader title="Emergency Logs" description="Emergency call event logs" actions={<Button onClick={() => setOpen(true)}>Add Emergency Logs</Button>} />
      <DataTable resource="/emergency_logs" columns={columns} perPage={20} reloadKey={reloadKey} searchPlaceholder="Search emergency logs…" emptyTitle="No emergency logs" emptyDescription="Create via Add button" />
      <FormDialog open={open} onOpenChange={setOpen} title="Add Emergency Logs" onSubmit={create} pending={pending} submitLabel="Create" pendingLabel="Creating…">
        <div><Label>Extension *</Label><Input value={form.extension} onChange={e=>setForm({...form,extension:e.target.value})} placeholder="1001" /></div>
        <div><Label>Event</Label><Input value={form.event} onChange={e=>setForm({...form,event:e.target.value})} placeholder="emergency" /></div>
      </FormDialog>
    </>
  );
}
