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
import { GitBranch } from "lucide-react";

export function CallFlows() {
  const [reloadKey, setReloadKey] = React.useState(0);
  const reload = () => setReloadKey((k) => k + 1);
  const [open, setOpen] = React.useState(false);
  const [form,setForm]=React.useState({ call_flow_name: "*90", description: "" });
  const columns: Column<any>[] = [
    { key: "call_flow_name", header: "Name", className: "num", cell: (r: any) => r["call_flow_name"] ?? r.name ?? JSON.stringify(r).slice(0, 60) },
    { key: "description", header: "Description", cell: (r:any) => (r as any).description || (r as any)["description"] || "—" }
  ];
  const createFn = async () => {
    try {
      await apiFetch("/call_flows", { method: "POST", body: JSON.stringify(form) });
      toast("Call flow created", { variant: "success" });
      setOpen(false);
      reload();
    } catch (err) {
      toastError(err);
    }
  };
  const { run: create, pending } = useAsyncAction(createFn);
  return (
    <>
      <PageHeader title="Call Flows" description="Feature code call flows" actions={<Button onClick={() => setOpen(true)}>Add Call Flows</Button>} />
      <DataTable
        resource="/call_flows"
        columns={columns}
        reloadKey={reloadKey}
        perPage={20}
        searchPlaceholder="Search call flows…"
        emptyTitle="No call flows"
        emptyDescription="Create your first call flows entry"
        emptyIcon={GitBranch}
        emptyAction={<Button onClick={() => setOpen(true)}><GitBranch className="h-4 w-4" /> Add Call Flows</Button>}
      />
      <FormDialog open={open} onOpenChange={setOpen} title="Add Call Flows" onSubmit={create} pending={pending} submitLabel="Create" pendingLabel="Creating…">
        <div><Label>Name *</Label><Input value={form.call_flow_name} onChange={e=>setForm({...form,call_flow_name:e.target.value})} placeholder="*90" /></div>
        <div><Label>Description</Label><Input value={form.description} onChange={e=>setForm({...form,description:e.target.value})} /></div>
      </FormDialog>
    </>
  );
}
