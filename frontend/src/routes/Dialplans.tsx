import * as React from "react";
import { apiFetch } from "../lib/api";
import { DataTable } from "../components/DataTable";
import type { Column } from "../components/types";
import { FormDialog } from "../components/FormDialog";
import { PageHeader } from "../components/shell/PageHeader";
import { StatusBadge } from "../components/StatusBadge";
import { Button } from "../components/ui/button";
import { Input } from "../components/ui/input";
import { Label } from "../components/ui/label";
import { Card, CardContent, CardHeader, CardTitle } from "../components/ui/card";
import { useConfirm } from "../components/ui/confirm";
import { toast, toastError } from "../lib/toast";
import { useAsyncAction } from "../lib/useAsyncAction";
import { GitBranch } from "lucide-react";

type DialplanRow = {
  dialplan_uuid: string;
  name: string;
  context: string;
  enabled: boolean;
  details?: any[];
};

export function Dialplans() {
  const confirm = useConfirm();
  const [reloadKey, setReloadKey] = React.useState(0);
  const reload = () => setReloadKey((k) => k + 1);
  const [open, setOpen] = React.useState(false);
  const [editing, setEditing] = React.useState<DialplanRow | null>(null);
  const [form, setForm] = React.useState({ name: "", context: "", description: "", dialplan_number: "", dialplan_continue: false, dialplan_order: "", hostname: "", dialplan_destination: false, destination: "^1000$", action: "transfer", data: "1000 XML $${domain}", enabled: true });

  const columns: Column<DialplanRow>[] = [
    { key: "name", header: "Name", className: "font-mono" },
    { key: "context", header: "Context" },
    {
      key: "enabled",
      header: "Status",
      cell: (r) => <StatusBadge tone={r.enabled ? "ok" : "off"}>{r.enabled ? "enabled" : "disabled"}</StatusBadge>,
    },
  ];

  const startCreate = () => { setEditing(null); setForm({ name: "", context: "", description: "", dialplan_number: "", dialplan_continue: false, dialplan_order: "", hostname: "", dialplan_destination: false, destination: "^1000$", action: "transfer", data: "1000 XML $${domain}", enabled: true }); setOpen(true); };
  const startEdit = (d: DialplanRow) => {
    setEditing(d);
    const details = (d as any).details || [];
    const cond = details.find((x: any) => x.tag === "condition");
    const act = details.find((x: any) => x.tag === "action");
    setForm({
      name: d.name || "",
      context: d.context || "",
      description: (d as any).description || "",
      dialplan_number: (d as any).dialplan_number || "",
      dialplan_continue: !!(d as any).dialplan_continue,
      dialplan_order: (d as any).dialplan_order ?? "",
      hostname: (d as any).hostname || "",
      dialplan_destination: !!(d as any).dialplan_destination,
      destination: cond?.data || "^1000$",
      action: act?.type || "transfer",
      data: act?.data || "1000 XML $${domain}",
      enabled: !!d.enabled,
    });
    setOpen(true);
  };

  const saveFn = async () => {
    try {
      const optNum = (v: string | number) => (v === "" ? undefined : Number(v));
      const extra = {
        description: form.description || undefined,
        dialplan_number: form.dialplan_number || undefined,
        dialplan_continue: form.dialplan_continue,
        dialplan_order: optNum(form.dialplan_order),
        hostname: form.hostname || undefined,
        dialplan_destination: form.dialplan_destination,
      };
      if (editing) {
        await apiFetch(`/dialplans/${editing.dialplan_uuid}`, { method: "PATCH", body: JSON.stringify({ name: form.name, context: form.context, enabled: form.enabled, ...extra }) });
        toast(`Dialplan ${form.name} updated`, { variant: "success" });
      } else {
        let domain = form.context;
        if (!domain) {
          const me: any = await apiFetch("/auth/me");
          if (me.domain_id) {
            const d: any = await apiFetch(`/domains/${me.domain_id}`);
            domain = d.domain_name;
          }
        }
        await apiFetch("/dialplans", { method: "POST", body: JSON.stringify({ name: form.name, context: domain, enabled: form.enabled, ...extra, details: [{ tag: "condition", type: "destination_number", data: form.destination }, { tag: "action", type: form.action, data: form.data }] }) });
        toast(`Dialplan ${form.name} created`, { variant: "success" });
      }
      setOpen(false); setEditing(null); reload();
    } catch (err) {
      toastError(err);
    }
  };
  const { run: save, pending: saving } = useAsyncAction(saveFn);

  const toggleEnabled = async (d: DialplanRow) => {
    try {
      await apiFetch(`/dialplans/${d.dialplan_uuid}`, { method: "PATCH", body: JSON.stringify({ enabled: !d.enabled }) });
      toast(`Dialplan ${d.name} ${!d.enabled ? "enabled" : "disabled"}`);
      reload();
    } catch (err) { toastError(err); }
  };
  const deleteDialplan = async (d: DialplanRow) => {
    const ok = await confirm({ title: `Delete dialplan ${d.name}?`, description: "This cannot be undone.", confirmLabel: "Delete" });
    if (!ok) return;
    try {
      await apiFetch(`/dialplans/${d.dialplan_uuid}`, { method: "DELETE" });
      toast(`Dialplan ${d.name} deleted`);
      reload();
    } catch (err) { toastError(err); }
  };

  return (
    <>
      <PageHeader
        title="Call Routing"
        description="Dialplans and call routing rules"
        actions={<Button onClick={startCreate}>Add Dialplan</Button>}
      />
      <DataTable<DialplanRow>
        resource="/dialplans"
        columns={columns}
        rowKey={(r) => r.dialplan_uuid}
        reloadKey={reloadKey}
        perPage={20}
        searchPlaceholder="Search dialplans…"
        emptyTitle="No dialplans"
        emptyDescription="Create your first call routing rule"
        emptyIcon={GitBranch}
        emptyAction={<Button onClick={startCreate}><GitBranch className="h-4 w-4" /> Add Dialplan</Button>}
        rowActions={(d) => (
          <div className="flex justify-end gap-1">
            <Button variant="ghost" size="sm" onClick={() => startEdit(d)}>Edit</Button>
            <Button variant="ghost" size="sm" onClick={() => toggleEnabled(d)}>{d.enabled ? "Disable" : "Enable"}</Button>
            <Button variant="ghost" size="sm" className="text-destructive hover:text-destructive" onClick={() => deleteDialplan(d)}>Delete</Button>
          </div>
        )}
      />
      <Card>
        <CardHeader><CardTitle>Wizard: Extensions → Trunks → Routes</CardTitle></CardHeader>
        <CardContent className="text-sm text-muted-foreground">
          Cudatel/Barracuda inspired 3-step: 1) Pick extension 1000, 2) Pick gateway, 3) Test dial — validates transfer.
        </CardContent>
      </Card>

      <FormDialog open={open} onOpenChange={(v) => { setOpen(v); if (!v) setEditing(null); }} title={editing ? "Edit Dialplan" : "Add Dialplan"} onSubmit={save} pending={saving} submitLabel={editing ? "Save" : "Create"}>
        <div><Label>Name *</Label><Input value={form.name} onChange={(e) => setForm({ ...form, name: e.target.value })} placeholder="inbound-1000" /></div>
        <div><Label>Context</Label><Input value={form.context} onChange={(e) => setForm({ ...form, context: e.target.value })} placeholder="tenant.example.com" /></div>
        <div className="grid grid-cols-2 gap-2">
          <div><Label>Number</Label><Input value={String(form.dialplan_number)} onChange={(e) => setForm({ ...form, dialplan_number: e.target.value })} placeholder="1000" /></div>
          <div><Label>Order</Label><Input type="number" value={String(form.dialplan_order)} onChange={(e) => setForm({ ...form, dialplan_order: e.target.value })} placeholder="100" /></div>
        </div>
        <div><Label>Description</Label><Input value={form.description} onChange={(e) => setForm({ ...form, description: e.target.value })} /></div>
        <div><Label>Hostname</Label><Input value={form.hostname} onChange={(e) => setForm({ ...form, hostname: e.target.value })} placeholder="fs1" /></div>
        <div className="flex items-center gap-4">
          <label className="flex items-center gap-1 text-xs"><input type="checkbox" checked={form.dialplan_continue} onChange={(e) => setForm({ ...form, dialplan_continue: e.target.checked })} /> Continue</label>
          <label className="flex items-center gap-1 text-xs"><input type="checkbox" checked={form.dialplan_destination} onChange={(e) => setForm({ ...form, dialplan_destination: e.target.checked })} /> Destination</label>
        </div>
        {!editing && <>
          <div><Label>Destination regex</Label><Input value={form.destination} onChange={(e) => setForm({ ...form, destination: e.target.value })} /></div>
          <div className="grid grid-cols-2 gap-2"><div><Label>Action</Label><Input value={form.action} onChange={(e) => setForm({ ...form, action: e.target.value })} /></div><div><Label>Data</Label><Input value={form.data} onChange={(e) => setForm({ ...form, data: e.target.value })} /></div></div>
        </>}
        <div className="flex items-center gap-2"><input type="checkbox" id="dp-enabled" checked={form.enabled} onChange={(e) => setForm({ ...form, enabled: e.target.checked })} className="h-4 w-4" /><Label htmlFor="dp-enabled" className="cursor-pointer">{form.enabled ? "Enabled" : "Disabled"}</Label></div>
      </FormDialog>
    </>
  );
}
