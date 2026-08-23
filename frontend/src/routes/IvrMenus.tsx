import * as React from "react";
import { apiFetch } from "../lib/api";
import { DataTable } from "../components/DataTable";
import type { Column } from "../components/types";
import { PageHeader } from "../components/shell/PageHeader";
import { StatusBadge } from "../components/StatusBadge";
import { Button } from "../components/ui/button";
import { Input } from "../components/ui/input";
import { Label } from "../components/ui/label";
import { FormDialog } from "../components/FormDialog";
import { useConfirm } from "../components/ui/confirm";
import { toast, toastError } from "../lib/toast";
import { useAsyncAction } from "../lib/useAsyncAction";
import { Phone } from "lucide-react";

export function IvrMenus() {
  const confirm = useConfirm();
  const [reloadKey, setReloadKey] = React.useState(0);
  const reload = () => setReloadKey((k) => k + 1);
  const [open, setOpen] = React.useState(false);
  const [editing, setEditing] = React.useState<any | null>(null);
  const [form, setForm] = React.useState({ name: "main-ivr", extension: "5000", options: "1:transfer:1000 XML $${domain},2:transfer:1001 XML $${domain}", enabled: true });

  const columns: Column<any>[] = [
    { key: "name", header: "Name", className: "font-mono" },
    { key: "extension", header: "Extension", className: "num" },
    { key: "enabled", header: "Status", cell: (r) => <StatusBadge tone={r.enabled ? "ok" : "off"}>{r.enabled ? "enabled" : "disabled"}</StatusBadge> },
  ];

  const startCreate = () => { setEditing(null); setForm({ name: "main-ivr", extension: "5000", options: "1:transfer:1000 XML $${domain},2:transfer:1001 XML $${domain}", enabled: true }); setOpen(true); };
  const startEdit = (m: any) => {
    setEditing(m);
    const opts = (m.options || []).map((o: any) => `${o.digits}:${o.action}:${o.param}`).join(",");
    setForm({ name: m.name || "", extension: m.extension || "", options: opts || "1:transfer:1000 XML $${domain}", enabled: !!m.enabled });
    setOpen(true);
  };

  const saveFn = async () => {
    try {
      const opts = form.options.split(",").map((s) => { const [digits, action, param] = s.split(":"); return { digits, action, param }; });
      if (editing) {
        await apiFetch(`/ivr_menus/${editing.ivr_menu_uuid}`, { method: "PATCH", body: JSON.stringify({ name: form.name, extension: form.extension, enabled: form.enabled, options: opts }) });
        toast(`IVR ${form.name} updated`, { variant: "success" });
      } else {
        await apiFetch("/ivr_menus", { method: "POST", body: JSON.stringify({ name: form.name, extension: form.extension, enabled: form.enabled, options: opts }) });
        toast(`IVR ${form.name} created`, { variant: "success" });
      }
      setOpen(false); setEditing(null); reload();
    } catch (err) {
      toastError(err);
    }
  };
  const { run: save, pending: saving } = useAsyncAction(saveFn);

  const toggleEnabled = async (m: any) => {
    try { await apiFetch(`/ivr_menus/${m.ivr_menu_uuid}`, { method: "PATCH", body: JSON.stringify({ enabled: !m.enabled }) }); toast(`IVR ${m.name} ${!m.enabled ? "enabled" : "disabled"}`); reload(); } catch (err) { toastError(err); }
  };
  const deleteIvr = async (m: any) => {
    const ok = await confirm({ title: `Delete IVR ${m.name}?`, description: "This cannot be undone.", confirmLabel: "Delete" });
    if (!ok) return;
    try { await apiFetch(`/ivr_menus/${m.ivr_menu_uuid}`, { method: "DELETE" }); toast(`IVR ${m.name} deleted`); reload(); } catch (err) { toastError(err); }
  };

  return (
    <>
      <PageHeader title="IVR Menus" description="Interactive voice response menus — digit maps to actions" actions={<Button onClick={startCreate}>Add IVR</Button>} />
      <DataTable
        resource="/ivr_menus"
        columns={columns}
        rowKey={(r) => r.ivr_menu_uuid}
        reloadKey={reloadKey}
        perPage={20}
        searchPlaceholder="Search IVRs…"
        emptyTitle="No IVR menus"
        emptyDescription="Create your first IVR menu"
        emptyIcon={Phone}
        emptyAction={<Button onClick={startCreate}><Phone className="h-4 w-4" /> Add IVR</Button>}
        rowActions={(m) => (
          <div className="flex justify-end gap-1">
            <Button variant="ghost" size="sm" onClick={() => startEdit(m)}>Edit</Button>
            <Button variant="ghost" size="sm" onClick={() => toggleEnabled(m)}>{m.enabled ? "Disable" : "Enable"}</Button>
            <Button variant="ghost" size="sm" className="text-destructive hover:text-destructive" onClick={() => deleteIvr(m)}>Delete</Button>
          </div>
        )}
      />
      <FormDialog open={open} onOpenChange={(v) => { setOpen(v); if (!v) setEditing(null); }} title={editing ? "Edit IVR" : "Add IVR"} onSubmit={save} pending={saving} submitLabel={editing ? "Save" : "Create"}>
        <div><Label>Name *</Label><Input value={form.name} onChange={(e) => setForm({ ...form, name: e.target.value })} /></div>
        <div><Label>Extension</Label><Input value={form.extension} onChange={(e) => setForm({ ...form, extension: e.target.value })} /></div>
        <div><Label>Options (digits:action:param comma separated)</Label><Input value={form.options} onChange={(e) => setForm({ ...form, options: e.target.value })} /></div>
        <div className="flex items-center gap-2"><input type="checkbox" id="ivr-enabled" checked={form.enabled} onChange={(e) => setForm({ ...form, enabled: e.target.checked })} className="h-4 w-4" /><Label htmlFor="ivr-enabled" className="cursor-pointer">{form.enabled ? "Enabled" : "Disabled"}</Label></div>
      </FormDialog>
    </>
  );
}
