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
import { MessageSquare } from "lucide-react";

export function Phrases() {
  const [reloadKey, setReloadKey] = React.useState(0);
  const reload = () => setReloadKey((k) => k + 1);
  const [open, setOpen] = React.useState(false);
  const [form,setForm]=React.useState({ phrase_name: "test-phrase", description: "" });
  const columns: Column<any>[] = [
    { key: "phrase_name", header: "Name", className: "num", cell: (r: any) => r["phrase_name"] ?? r.name ?? JSON.stringify(r).slice(0, 60) },
    { key: "description", header: "Description", cell: (r:any) => (r as any).description || (r as any)["description"] || "—" }
  ];
  const createFn = async () => {
    try {
      await apiFetch("/phrases", { method: "POST", body: JSON.stringify(form) });
      toast("Phrase created", { variant: "success" });
      setOpen(false);
      reload();
    } catch (err) {
      toastError(err);
    }
  };
  const { run: create, pending } = useAsyncAction(createFn);
  return (
    <>
      <PageHeader title="Phrases" description="Phrase macros" actions={<Button onClick={() => setOpen(true)}>Add Phrases</Button>} />
      <DataTable
        resource="/phrases"
        columns={columns}
        reloadKey={reloadKey}
        perPage={20}
        searchPlaceholder="Search phrases…"
        emptyTitle="No phrases"
        emptyDescription="Create your first phrases entry"
        emptyIcon={MessageSquare}
        emptyAction={<Button onClick={() => setOpen(true)}><MessageSquare className="h-4 w-4" /> Add Phrases</Button>}
      />
      <FormDialog open={open} onOpenChange={setOpen} title="Add Phrases" onSubmit={create} pending={pending} submitLabel="Create" pendingLabel="Creating…">
        <div><Label>Name *</Label><Input value={form.phrase_name} onChange={e=>setForm({...form,phrase_name:e.target.value})} placeholder="test-phrase" /></div>
        <div><Label>Description</Label><Input value={form.description} onChange={e=>setForm({...form,description:e.target.value})} /></div>
      </FormDialog>
    </>
  );
}
