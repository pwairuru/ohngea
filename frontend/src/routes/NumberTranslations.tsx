import * as React from "react";
import { apiFetch } from "../lib/api";
import { DataTable } from "../components/DataTable";
import type { Column } from "../components/types";
import { PageHeader } from "../components/shell/PageHeader";
import { Button } from "../components/ui/button";
import { Label } from "../components/ui/label";
import { FormDialog } from "../components/FormDialog";
import { toast, toastError } from "../lib/toast";
import { useAsyncAction } from "../lib/useAsyncAction";
import { ArrowLeftRight } from "lucide-react";

export function NumberTranslations() {
  const [reloadKey, setReloadKey] = React.useState(0);
  const reload = () => setReloadKey((k) => k + 1);
  const [open, setOpen] = React.useState(false);
  const [json, setJson] = React.useState(`{"number_translation_name": "test-nt"}`);
  const columns: Column<any>[] = [
    { key: "number_translation_name", header: "Name", className: "font-mono text-xs", cell: (r: any) => r["number_translation_name"] ?? r.name ?? JSON.stringify(r).slice(0, 60) },
    { key: "details", header: "Details", cell: (r: any) => <span className="text-muted-foreground text-xs break-all">{JSON.stringify(r).slice(0, 80)}</span> },
  ];
  const createFn = async () => {
    try {
      const body = JSON.parse(json);
      await apiFetch("/number_translations", { method: "POST", body: JSON.stringify(body) });
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
        <div><Label>JSON payload *</Label><textarea value={json} onChange={(e) => setJson(e.target.value)} rows={6} className="w-full rounded border p-2 font-mono text-xs" /></div>
        <div className="text-xs text-muted-foreground">Required field shown above — edit JSON and press Create.</div>
      </FormDialog>
    </>
  );
}
