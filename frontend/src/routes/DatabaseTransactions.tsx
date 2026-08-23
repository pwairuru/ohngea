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

export function DatabaseTransactions() {
  const [open, setOpen] = React.useState(false);
  const [form,setForm]=React.useState({ app_name: "extensions", transaction_code: "add" });
  const [reloadKey, setReloadKey] = React.useState(0);
  const reload = () => setReloadKey((k) => k + 1);

  const { run: create, pending } = useAsyncAction(async () => {
    try {
      await apiFetch("/database_transactions", { method: "POST", body: JSON.stringify(form) });
      toast("Database transaction created", { variant: "success" });
      setOpen(false);
      reload();
    } catch (err) {
      toastError(err);
    }
  });

  const columns: Column<any>[] = [
    { key: "app_name", header: "Name", className: "num", cell: (r: any) => r["app_name"] ?? r["transaction_code"] ?? r.name ?? JSON.stringify(r).slice(0,60) },
    { key: "transaction_code", header: "Code", cell: (r:any) => (r as any).transaction_code || (r as any)["transaction_code"] || "—" }
  ];

  return (
    <>
      <PageHeader title="Database Transactions" description="Database transactions and jobs" actions={<Button onClick={() => setOpen(true)}>Add Database Transactions</Button>} />
      <DataTable resource="/database_transactions" columns={columns} perPage={20} reloadKey={reloadKey} searchPlaceholder="Search database transactions…" emptyTitle="No database transactions" emptyDescription="Create via Add button" />
      <FormDialog open={open} onOpenChange={setOpen} title="Add Database Transactions" onSubmit={create} pending={pending} submitLabel="Create" pendingLabel="Creating…">
        <div><Label>App *</Label><Input value={form.app_name} onChange={e=>setForm({...form,app_name:e.target.value})} placeholder="extensions" /></div>
        <div><Label>Code</Label><Input value={form.transaction_code} onChange={e=>setForm({...form,transaction_code:e.target.value})} placeholder="add" /></div>
      </FormDialog>
    </>
  );
}
