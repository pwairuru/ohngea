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
import { Hash } from "lucide-react";

export function FeatureCodes() {
  const [reloadKey, setReloadKey] = React.useState(0);
  const reload = () => setReloadKey((k) => k + 1);
  const [open, setOpen] = React.useState(false);
  const [form,setForm]=React.useState({ feature_code_name: "*99", description: "" });
  const columns: Column<any>[] = [
    { key: "feature_code_name", header: "Code", className: "num", cell: (r: any) => r["feature_code_name"] ?? r.name ?? JSON.stringify(r).slice(0, 60) },
    { key: "description", header: "Description", cell: (r:any) => (r as any).description || (r as any)["description"] || "—" }
  ];
  const createFn = async () => {
    try {
      await apiFetch("/feature_codes", { method: "POST", body: JSON.stringify(form) });
      toast("Feature code created", { variant: "success" });
      setOpen(false);
      reload();
    } catch (err) {
      toastError(err);
    }
  };
  const { run: create, pending } = useAsyncAction(createFn);
  return (
    <>
      <PageHeader title="Feature Codes" description="Star codes and feature toggles" actions={<Button onClick={() => setOpen(true)}>Add Feature Codes</Button>} />
      <DataTable
        resource="/feature_codes"
        columns={columns}
        reloadKey={reloadKey}
        perPage={20}
        searchPlaceholder="Search feature codes…"
        emptyTitle="No feature codes"
        emptyDescription="Create your first feature codes entry"
        emptyIcon={Hash}
        emptyAction={<Button onClick={() => setOpen(true)}><Hash className="h-4 w-4" /> Add Feature Codes</Button>}
      />
      <FormDialog open={open} onOpenChange={setOpen} title="Add Feature Codes" onSubmit={create} pending={pending} submitLabel="Create" pendingLabel="Creating…">
        <div><Label>Code *</Label><Input value={form.feature_code_name} onChange={e=>setForm({...form,feature_code_name:e.target.value})} placeholder="*99" /></div>
        <div><Label>Description</Label><Input value={form.description} onChange={e=>setForm({...form,description:e.target.value})} /></div>
      </FormDialog>
    </>
  );
}
