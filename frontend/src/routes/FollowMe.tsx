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
import { UserCheck } from "lucide-react";

export function FollowMe() {
  const [reloadKey, setReloadKey] = React.useState(0);
  const reload = () => setReloadKey((k) => k + 1);
  const [open, setOpen] = React.useState(false);
  const [form,setForm]=React.useState({ follow_me_number: "1001", description: "" });
  const columns: Column<any>[] = [
    { key: "follow_me_number", header: "Number", className: "num", cell: (r: any) => r["follow_me_number"] ?? r.name ?? JSON.stringify(r).slice(0, 60) },
    { key: "description", header: "Description", cell: (r:any) => (r as any).description || (r as any)["description"] || "—" }
  ];
  const createFn = async () => {
    try {
      await apiFetch("/follow_me", { method: "POST", body: JSON.stringify(form) });
      toast("Follow me created", { variant: "success" });
      setOpen(false);
      reload();
    } catch (err) {
      toastError(err);
    }
  };
  const { run: create, pending } = useAsyncAction(createFn);
  return (
    <>
      <PageHeader title="Follow Me" description="Find-me / follow-me entries" actions={<Button onClick={() => setOpen(true)}>Add Follow Me</Button>} />
      <DataTable
        resource="/follow_me"
        columns={columns}
        reloadKey={reloadKey}
        perPage={20}
        searchPlaceholder="Search follow me…"
        emptyTitle="No follow me"
        emptyDescription="Create your first follow me entry"
        emptyIcon={UserCheck}
        emptyAction={<Button onClick={() => setOpen(true)}><UserCheck className="h-4 w-4" /> Add Follow Me</Button>}
      />
      <FormDialog open={open} onOpenChange={setOpen} title="Add Follow Me" onSubmit={create} pending={pending} submitLabel="Create" pendingLabel="Creating…">
        <div><Label>Number *</Label><Input value={form.follow_me_number} onChange={e=>setForm({...form,follow_me_number:e.target.value})} placeholder="1001" /></div>
        <div><Label>Description</Label><Input value={form.description} onChange={e=>setForm({...form,description:e.target.value})} /></div>
      </FormDialog>
    </>
  );
}
