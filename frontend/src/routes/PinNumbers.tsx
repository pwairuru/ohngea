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
import { KeyRound } from "lucide-react";

export function PinNumbers() {
  const [reloadKey, setReloadKey] = React.useState(0);
  const reload = () => setReloadKey((k) => k + 1);
  const [open, setOpen] = React.useState(false);
  const [form,setForm]=React.useState({ accountcode: "1234", pin_number: "1234" });
  const columns: Column<any>[] = [
    { key: "pin_number", header: "PIN", className: "num", cell: (r: any) => r["pin_number"] ?? r.name ?? JSON.stringify(r).slice(0, 60) },
    { key: "accountcode", header: "Account Code", cell: (r:any) => (r as any).accountcode || (r as any)["accountcode"] || "—" }
  ];
  const createFn = async () => {
    try {
      await apiFetch("/pin_numbers", { method: "POST", body: JSON.stringify(form) });
      toast("PIN created", { variant: "success" });
      setOpen(false);
      reload();
    } catch (err) {
      toastError(err);
    }
  };
  const { run: create, pending } = useAsyncAction(createFn);
  return (
    <>
      <PageHeader title="Pin Numbers" description="PIN codes for auth" actions={<Button onClick={() => setOpen(true)}>Add Pin Numbers</Button>} />
      <DataTable
        resource="/pin_numbers"
        columns={columns}
        reloadKey={reloadKey}
        perPage={20}
        searchPlaceholder="Search pin numbers…"
        emptyTitle="No pin numbers"
        emptyDescription="Create your first pin numbers entry"
        emptyIcon={KeyRound}
        emptyAction={<Button onClick={() => setOpen(true)}><KeyRound className="h-4 w-4" /> Add Pin Numbers</Button>}
      />
      <FormDialog open={open} onOpenChange={setOpen} title="Add Pin Numbers" onSubmit={create} pending={pending} submitLabel="Create" pendingLabel="Creating…">
        <div><Label>Account Code *</Label><Input value={form.accountcode} onChange={e=>setForm({...form,accountcode:e.target.value})} placeholder="1234" /></div>
        <div><Label>PIN</Label><Input value={form.pin_number} onChange={e=>setForm({...form,pin_number:e.target.value})} placeholder="1234" /></div>
      </FormDialog>
    </>
  );
}
