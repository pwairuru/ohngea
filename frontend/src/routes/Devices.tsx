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
import { Card, CardContent } from "../components/ui/card";
import { useConfirm } from "../components/ui/confirm";
import { toast, toastError } from "../lib/toast";
import { useAsyncAction } from "../lib/useAsyncAction";
import { Smartphone } from "lucide-react";

type Device = {
  device_uuid: string;
  mac: string;
  vendor: string;
  model: string;
  label: string;
  enabled: boolean;
};

export function Devices() {
  const confirm = useConfirm();
  const [reloadKey, setReloadKey] = React.useState(0);
  const reload = () => setReloadKey((k) => k + 1);
  const [open, setOpen] = React.useState(false);
  const [editing, setEditing] = React.useState<Device | null>(null);
  const [form, setForm] = React.useState({ mac: "00:11:22:33:44:55", vendor: "poly", model: "VVX250", label: "", enabled: true });
  const [provision, setProvision] = React.useState<string>("");

  const columns: Column<Device>[] = [
    { key: "mac", header: "MAC", className: "num font-mono", sortable: false },
    { key: "vendor", header: "Vendor" },
    { key: "model", header: "Model" },
    {
      key: "enabled",
      header: "Status",
      cell: (r) => <StatusBadge tone={r.enabled ? "ok" : "off"}>{r.enabled ? "enabled" : "disabled"}</StatusBadge>,
    },
  ];

  const startCreate = () => {
    setEditing(null);
    setForm({ mac: "00:11:22:33:44:55", vendor: "poly", model: "VVX250", label: "", enabled: true });
    setOpen(true);
  };
  const startEdit = (d: Device) => {
    setEditing(d);
    setForm({ mac: d.mac || "", vendor: d.vendor || "poly", model: d.model || "", label: (d as any).label || "", enabled: !!d.enabled });
    setOpen(true);
  };

  const saveFn = async () => {
    try {
      if (editing) {
        const patchBody: Record<string, unknown> = { mac: form.mac, enabled: form.enabled };
        for (const [key, value] of Object.entries(form)) {
          if (key === "mac" || key === "enabled") continue;
          if (value !== "") patchBody[key] = value;
        }
        await apiFetch(`/devices/${editing.device_uuid}`, { method: "PATCH", body: JSON.stringify(patchBody) });
        toast(`Device ${form.mac} updated`, { variant: "success" });
      } else {
        await apiFetch("/devices", { method: "POST", body: JSON.stringify(form) });
        toast(`Device ${form.mac} created`, { variant: "success" });
      }
      setOpen(false);
      setEditing(null);
      reload();
    } catch (err) {
      toastError(err);
    }
  };
  const { run: save, pending: saving } = useAsyncAction(saveFn);

  const toggleEnabled = async (d: Device) => {
    try {
      await apiFetch(`/devices/${d.device_uuid}`, { method: "PATCH", body: JSON.stringify({ enabled: !d.enabled }) });
      toast(`Device ${d.mac} ${!d.enabled ? "enabled" : "disabled"}`, { variant: "success" });
      reload();
    } catch (err) {
      toastError(err);
    }
  };

  const deleteDevice = async (d: Device) => {
    const ok = await confirm({ title: `Delete device ${d.mac}?`, description: "This cannot be undone.", confirmLabel: "Delete" });
    if (!ok) return;
    try {
      await apiFetch(`/devices/${d.device_uuid}`, { method: "DELETE" });
      toast(`Device ${d.mac} deleted`);
      reload();
    } catch (err) {
      toastError(err);
    }
  };

  const preview = async (mac: string, vendor: string) => {
    try {
      const txt = await fetch(`/provision/${encodeURIComponent(mac)}.cfg`).then((r) => r.text());
      setProvision(txt);
    } catch {
      setProvision(`# Provision for ${vendor} mac ${mac}\n(preview fallback)`);
    }
  };

  return (
    <>
      <PageHeader
        title="Devices"
        description="Provisioned SIP devices and phone configs"
        actions={<Button onClick={startCreate}>Add Device</Button>}
      />
      <DataTable<Device>
        resource="/devices"
        columns={columns}
        rowKey={(r) => r.device_uuid}
        reloadKey={reloadKey}
        perPage={20}
        searchPlaceholder="Search devices…"
        emptyTitle="No devices"
        emptyDescription="Add your first provisioned device"
        emptyIcon={Smartphone}
        emptyAction={<Button onClick={startCreate}><Smartphone className="h-4 w-4" /> Add Device</Button>}
        rowActions={(d) => (
          <div className="flex justify-end gap-1">
            <Button variant="ghost" size="sm" onClick={() => preview(d.mac, d.vendor)}>Preview</Button>
            <Button variant="ghost" size="sm" onClick={() => startEdit(d)}>Edit</Button>
            <Button variant="ghost" size="sm" onClick={() => toggleEnabled(d)}>{d.enabled ? "Disable" : "Enable"}</Button>
            <Button variant="ghost" size="sm" className="text-destructive hover:text-destructive" onClick={() => deleteDevice(d)}>Delete</Button>
          </div>
        )}
      />
      {provision && (
        <Card>
          <CardContent className="pt-4">
            <pre className="p-3 bg-muted rounded text-xs overflow-auto max-h-48">{provision}</pre>
            <Button variant="outline" className="mt-2" onClick={() => setProvision("")}>Close</Button>
          </CardContent>
        </Card>
      )}

      <FormDialog
        open={open}
        onOpenChange={(v) => { setOpen(v); if (!v) setEditing(null); }}
        title={editing ? "Edit Device" : "Add Device"}
        onSubmit={save}
        pending={saving}
        submitLabel={editing ? "Save" : "Create"}
      >
        <div><Label>MAC *</Label><Input value={form.mac} onChange={(e) => setForm({ ...form, mac: e.target.value })} placeholder="00:11:22:33:44:55" /></div>
        <div className="grid grid-cols-2 gap-2"><div><Label>Vendor</Label><Input value={form.vendor} onChange={(e) => setForm({ ...form, vendor: e.target.value })} placeholder="poly/yealink" /></div><div><Label>Model</Label><Input value={form.model} onChange={(e) => setForm({ ...form, model: e.target.value })} /></div></div>
        <div><Label>Label</Label><Input value={form.label} onChange={(e) => setForm({ ...form, label: e.target.value })} placeholder="Desk phone" /></div>
        <div className="flex items-center gap-2"><input type="checkbox" id="dev-enabled" checked={form.enabled} onChange={(e) => setForm({ ...form, enabled: e.target.checked })} className="h-4 w-4" /><Label htmlFor="dev-enabled" className="cursor-pointer">{form.enabled ? "Enabled" : "Disabled"}</Label></div>
      </FormDialog>
    </>
  );
}
