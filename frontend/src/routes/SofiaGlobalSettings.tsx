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

export function SofiaGlobalSettings() {
  const [open, setOpen] = React.useState(false);
  const [form,setForm]=React.useState({ global_setting_name: "sip-trace", global_setting_value: "false" });
  const [reloadKey, setReloadKey] = React.useState(0);
  const reload = () => setReloadKey((k) => k + 1);

  const { run: create, pending } = useAsyncAction(async () => {
    try {
      await apiFetch("/sofia_global_settings", { method: "POST", body: JSON.stringify(form) });
      toast("Sofia setting created", { variant: "success" });
      setOpen(false);
      reload();
    } catch (err) {
      toastError(err);
    }
  });

  const columns: Column<any>[] = [
    { key: "global_setting_name", header: "Name", className: "num", cell: (r: any) => r["global_setting_name"] ?? r["global_setting_value"] ?? r.name ?? JSON.stringify(r).slice(0,60) },
    { key: "global_setting_value", header: "Value", cell: (r:any) => (r as any).global_setting_value || (r as any)["global_setting_value"] || "—" }
  ];

  return (
    <>
      <PageHeader title="Sofia Global Settings" description="Global Sofia SIP stack tuning" actions={<Button onClick={() => setOpen(true)}>Add Sofia Global Settings</Button>} />
      <DataTable resource="/sofia_global_settings" columns={columns} perPage={20} reloadKey={reloadKey} searchPlaceholder="Search sofia global settings…" emptyTitle="No sofia global settings" emptyDescription="Create via Add button" />
      <FormDialog open={open} onOpenChange={setOpen} title="Add Sofia Global Settings" onSubmit={create} pending={pending} submitLabel="Create" pendingLabel="Creating…">
        <div><Label>Name *</Label><Input value={form.global_setting_name} onChange={e=>setForm({...form,global_setting_name:e.target.value})} placeholder="sip-trace" /></div>
        <div><Label>Value</Label><Input value={form.global_setting_value} onChange={e=>setForm({...form,global_setting_value:e.target.value})} placeholder="false" /></div>
      </FormDialog>
    </>
  );
}
