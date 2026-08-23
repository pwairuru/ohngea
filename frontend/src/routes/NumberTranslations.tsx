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
import { ArrowLeftRight } from "lucide-react";

export function NumberTranslations() {
  const [reloadKey, setReloadKey] = React.useState(0);
  const reload = () => setReloadKey((k) => k + 1);
  const [open, setOpen] = React.useState(false);
  const [form,setForm]=React.useState({ number_translation_name: "test-nt", description: "" });
  const columns: Column<any>[] = [
    { key: "number_translation_name", header: "Name", className: "num", cell: (r: any) => r["number_translation_name"] ?? r.name ?? JSON.stringify(r).slice(0, 60) },
    { key: "description", header: "Description", cell: (r:any) => (r as any).description || (r as any)["description"] || "—" }
  ];
  const createFn = async () => {
    try {
      await apiFetch("/number_translations", { method: "POST", body: JSON.stringify(form) });
      toast("Number translation created", { variant: "success" });
      setOpen(false);
      reload();
    } catch (err) {
      toastError(err);
    }
  };
  const { run: create, pending } = useAsyncAction(createFn);
  return (
    <>
      <PageHeader title="Number Translations" description="Number translation rules" actions={<Button onClick={() => setOpen(true)}>Add Number Translations</Button>} />
      <DataTable
        resource="/number_translations"
        columns={columns}
        reloadKey={reloadKey}
        perPage={20}
        searchPlaceholder="Search number translations…"
        emptyTitle="No number translations"
        emptyDescription="Create your first number translations entry"
        emptyIcon={ArrowLeftRight}
        emptyAction={<Button onClick={() => setOpen(true)}><ArrowLeftRight className="h-4 w-4" /> Add Number Translations</Button>}
      />
      <FormDialog open={open} onOpenChange={setOpen} title="Add Number Translations" onSubmit={create} pending={pending} submitLabel="Create" pendingLabel="Creating…">
        <div><Label>Name *</Label><Input value={form.number_translation_name} onChange={e=>setForm({...form,number_translation_name:e.target.value})} placeholder="test-nt" /></div>
        <div><Label>Description</Label><Input value={form.description} onChange={e=>setForm({...form,description:e.target.value})} /></div>
      </FormDialog>
    </>
  );
}
