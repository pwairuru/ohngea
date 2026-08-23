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
import { Hash } from "lucide-react";

export function FeatureCodes() {
  const [reloadKey, setReloadKey] = React.useState(0);
  const reload = () => setReloadKey((k) => k + 1);
  const [open, setOpen] = React.useState(false);
  const [json, setJson] = React.useState(`{"feature_code_name": "*99"}`);
  const columns: Column<any>[] = [
    { key: "feature_code_name", header: "Code", className: "font-mono text-xs", cell: (r: any) => r["feature_code_name"] ?? r.name ?? JSON.stringify(r).slice(0, 60) },
    { key: "details", header: "Details", cell: (r: any) => <span className="text-muted-foreground text-xs break-all">{JSON.stringify(r).slice(0, 80)}</span> },
  ];
  const createFn = async () => {
    try {
      const body = JSON.parse(json);
      await apiFetch("/feature_codes", { method: "POST", body: JSON.stringify(body) });
      toast("FeatureCodes created", { variant: "success" });
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
        <div><Label>JSON payload *</Label><textarea value={json} onChange={(e) => setJson(e.target.value)} rows={6} className="w-full rounded border p-2 font-mono text-xs" /></div>
        <div className="text-xs text-muted-foreground">Required field shown above — edit JSON and press Create.</div>
      </FormDialog>
    </>
  );
}
