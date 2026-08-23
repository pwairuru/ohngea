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

export function ExtensionSettings() {
  const [open, setOpen] = React.useState(false);
  const [form,setForm]=React.useState({ extension_name: "call_timeout", extension_value: "30" });
  const [reloadKey, setReloadKey] = React.useState(0);
  const reload = () => setReloadKey((k) => k + 1);

  const { run: create, pending } = useAsyncAction(async () => {
    try {
      await apiFetch("/extension_settings", { method: "POST", body: JSON.stringify(form) });
      toast("Extension setting created", { variant: "success" });
      setOpen(false);
      reload();
    } catch (err) {
      toastError(err);
    }
  });

  const columns: Column<any>[] = [
    { key: "extension_name", header: "Name", className: "num", cell: (r: any) => r["extension_name"] ?? r["extension_value"] ?? r.name ?? JSON.stringify(r).slice(0,60) },
    { key: "extension_value", header: "Value", cell: (r:any) => (r as any).extension_value || (r as any)["extension_value"] || "—" }
  ];

  return (
    <>
      <PageHeader title="Extension Settings" description="Per-extension override settings" actions={<Button onClick={() => setOpen(true)}>Add Extension Settings</Button>} />
      <DataTable resource="/extension_settings" columns={columns} perPage={20} reloadKey={reloadKey} searchPlaceholder="Search extension settings…" emptyTitle="No extension settings" emptyDescription="Create via Add button" />
      <FormDialog open={open} onOpenChange={setOpen} title="Add Extension Settings" onSubmit={create} pending={pending} submitLabel="Create" pendingLabel="Creating…">
        <div><Label>Name *</Label><Input value={form.extension_name} onChange={e=>setForm({...form,extension_name:e.target.value})} placeholder="call_timeout" /></div>
        <div><Label>Value</Label><Input value={form.extension_value} onChange={e=>setForm({...form,extension_value:e.target.value})} placeholder="30" /></div>
      </FormDialog>
    </>
  );
}
