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
import { MapPin } from "lucide-react";

export function Destinations() {
  const [reloadKey, setReloadKey] = React.useState(0);
  const reload = () => setReloadKey((k) => k + 1);
  const [open, setOpen] = React.useState(false);
  const [form,setForm]=React.useState({ destination_prefix: "1", destination_number: "1001" });
  const columns: Column<any>[] = [
    { key: "destination_number", header: "Number", className: "num", cell: (r: any) => r["destination_number"] ?? r.name ?? JSON.stringify(r).slice(0, 60) },
    { key: "destination_prefix", header: "Prefix", cell: (r:any) => (r as any).destination_prefix || (r as any)["destination_prefix"] || "—" }
  ];
  const createFn = async () => {
    try {
      await apiFetch("/destinations", { method: "POST", body: JSON.stringify(form) });
      toast("Destination created", { variant: "success" });
      setOpen(false);
      reload();
    } catch (err) {
      toastError(err);
    }
  };
  const { run: create, pending } = useAsyncAction(createFn);
  return (
    <>
      <PageHeader title="Destinations" description="Outbound routing destinations" actions={<Button onClick={() => setOpen(true)}>Add Destinations</Button>} />
      <DataTable
        resource="/destinations"
        columns={columns}
        reloadKey={reloadKey}
        perPage={20}
        searchPlaceholder="Search destinations…"
        emptyTitle="No destinations"
        emptyDescription="Create your first destinations entry"
        emptyIcon={MapPin}
        emptyAction={<Button onClick={() => setOpen(true)}><MapPin className="h-4 w-4" /> Add Destinations</Button>}
      />
      <FormDialog open={open} onOpenChange={setOpen} title="Add Destinations" onSubmit={create} pending={pending} submitLabel="Create" pendingLabel="Creating…">
        <div><Label>Prefix *</Label><Input value={form.destination_prefix} onChange={e=>setForm({...form,destination_prefix:e.target.value})} placeholder="1" /></div>
        <div><Label>Number</Label><Input value={form.destination_number} onChange={e=>setForm({...form,destination_number:e.target.value})} placeholder="1001" /></div>
      </FormDialog>
    </>
  );
}
