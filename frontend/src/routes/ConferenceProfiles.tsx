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

export function ConferenceProfiles() {
  const [open, setOpen] = React.useState(false);
  const [form,setForm]=React.useState({ profile_name: "conf-default", description: "" });
  const [reloadKey, setReloadKey] = React.useState(0);
  const reload = () => setReloadKey((k) => k + 1);

  const { run: create, pending } = useAsyncAction(async () => {
    try {
      await apiFetch("/conference_profiles", { method: "POST", body: JSON.stringify(form) });
      toast("Conference profile created", { variant: "success" });
      setOpen(false);
      reload();
    } catch (err) {
      toastError(err);
    }
  });

  const columns: Column<any>[] = [
    { key: "profile_name", header: "Name", className: "num", cell: (r: any) => r["profile_name"] ?? r.name ?? JSON.stringify(r).slice(0,60) },
    { key: "description", header: "Description", cell: (r:any) => (r as any).description || (r as any)["description"] || "—" }
  ];

  return (
    <>
      <PageHeader title="Conference Profiles" description="Conference profile parameters" actions={<Button onClick={() => setOpen(true)}>Add Conference Profiles</Button>} />
      <DataTable resource="/conference_profiles" columns={columns} perPage={20} reloadKey={reloadKey} searchPlaceholder="Search conference profiles…" emptyTitle="No conference profiles" emptyDescription="Create via Add button" />
      <FormDialog open={open} onOpenChange={setOpen} title="Add Conference Profiles" onSubmit={create} pending={pending} submitLabel="Create" pendingLabel="Creating…">
        <div><Label>Profile *</Label><Input value={form.profile_name} onChange={e=>setForm({...form,profile_name:e.target.value})} placeholder="conf-default" /></div>
        <div><Label>Description</Label><Input value={form.description} onChange={e=>setForm({...form,description:e.target.value})} /></div>
      </FormDialog>
    </>
  );
}
