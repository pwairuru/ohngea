import * as React from "react";
import { apiFetch } from "../lib/api";
import { DataTable } from "../components/DataTable";
import type { Column } from "../components/types";
import { PageHeader } from "../components/shell/PageHeader";
import { StatusBadge } from "../components/StatusBadge";
import { Button } from "../components/ui/button";
import { Input } from "../components/ui/input";
import { Label } from "../components/ui/label";
import { Card, CardContent, CardHeader, CardTitle } from "../components/ui/card";
import { FormDialog } from "../components/FormDialog";
import { useConfirm } from "../components/ui/confirm";
import { toast, toastError } from "../lib/toast";
import { useAsyncAction } from "../lib/useAsyncAction";
import { Building } from "lucide-react";

export function Domains() {
  const confirm = useConfirm();
  const [reloadKey, setReloadKey] = React.useState(0);
  const reload = () => setReloadKey((k) => k + 1);
  const [open, setOpen] = React.useState(false);
  const [editing, setEditing] = React.useState<any | null>(null);
  const [name, setName] = React.useState("tenant-" + Math.floor(Math.random() * 1000) + ".example.com");
  const [enabled, setEnabled] = React.useState(true);

  const columns: Column<any>[] = [
    { key: "domain_name", header: "Domain", className: "font-mono" },
    { key: "enabled", header: "Enabled", cell: (r) => <StatusBadge tone={r.enabled ? "ok" : "off"}>{r.enabled ? "active" : "disabled"}</StatusBadge> },
    { key: "created_at", header: "Created", className: "num", cell: (r) => r.created_at ? new Date(r.created_at).toLocaleDateString() : "-" },
  ];

  const startCreate = () => { setEditing(null); setName("tenant-" + Math.floor(Math.random() * 1000) + ".example.com"); setEnabled(true); setOpen(true); };
  const startEdit = (d: any) => { setEditing(d); setName(d.domain_name || ""); setEnabled(!!d.enabled); setOpen(true); };

  const saveFn = async () => {
    try {
      if (editing) {
        await apiFetch(`/domains/${editing.domain_id}`, { method: "PATCH", body: JSON.stringify({ domain_name: name, enabled }) });
        toast(`Domain ${name} updated`, { variant: "success" });
      } else {
        await apiFetch("/domains", { method: "POST", body: JSON.stringify({ domain_name: name, enabled }) });
        toast(`Domain ${name} created`, { variant: "success" });
      }
      setOpen(false); setEditing(null); reload();
    } catch (err) { toastError(err); }
  };
  const { run: save, pending: saving } = useAsyncAction(saveFn);

  const toggleEnabled = async (d: any) => {
    try { await apiFetch(`/domains/${d.domain_id}`, { method: "PATCH", body: JSON.stringify({ enabled: !d.enabled }) }); toast(`Domain ${d.domain_name} ${!d.enabled ? "enabled" : "disabled"}`); reload(); } catch (err) { toastError(err); }
  };
  const deleteDomain = async (d: any) => {
    const ok = await confirm({ title: `Delete domain ${d.domain_name}?`, description: "This will remove all tenant data.", confirmLabel: "Delete" });
    if (!ok) return;
    try { await apiFetch(`/domains/${d.domain_id}`, { method: "DELETE" }); toast(`Domain ${d.domain_name} deleted`); reload(); } catch (err) { toastError(err); }
  };

  return (
    <>
      <PageHeader title="Tenants (Domains)" description="Thousands tenants — hash partitioned" actions={<Button onClick={startCreate}>Add Tenant</Button>} />
      <Card>
        <CardHeader><CardTitle>Thousands tenants — hash partitioned</CardTitle></CardHeader>
        <CardContent className="text-sm text-muted-foreground">Tenants partitioned by domain_id, hash partitioned, via <code>GET /domains</code>. Seeded with dialplans + superadmin.</CardContent>
      </Card>
      <DataTable
        resource="/domains"
        columns={columns}
        rowKey={(r) => r.domain_id}
        reloadKey={reloadKey}
        perPage={20}
        searchPlaceholder="Search tenants…"
        emptyTitle="No tenants"
        emptyDescription="Create first via subscription"
        emptyIcon={Building}
        emptyAction={<Button onClick={startCreate}><Building className="h-4 w-4" /> Add Tenant</Button>}
        rowActions={(d) => (
          <div className="flex justify-end gap-1">
            <Button variant="ghost" size="sm" onClick={() => startEdit(d)}>Edit</Button>
            <Button variant="ghost" size="sm" onClick={() => toggleEnabled(d)}>{d.enabled ? "Disable" : "Enable"}</Button>
            <Button variant="ghost" size="sm" className="text-destructive hover:text-destructive" onClick={() => deleteDomain(d)}>Delete</Button>
          </div>
        )}
      />
      <FormDialog open={open} onOpenChange={(v) => { setOpen(v); if (!v) setEditing(null); }} title={editing ? "Edit Tenant" : "Add Tenant (Subscription)"} onSubmit={save} pending={saving} submitLabel={editing ? "Save" : "Create"}>
        <div><Label>Domain *</Label><Input value={name} onChange={(e) => setName(e.target.value)} placeholder="tenant.example.com" /></div>
        <div className="flex items-center gap-2"><input type="checkbox" id="domain-enabled" checked={enabled} onChange={(e) => setEnabled(e.target.checked)} className="h-4 w-4" /><Label htmlFor="domain-enabled" className="cursor-pointer">{enabled ? "Enabled" : "Disabled"}</Label></div>
        <div className="text-xs text-muted-foreground">Will seed dialplans + superadmin, attach hash partition.</div>
      </FormDialog>
    </>
  );
}
