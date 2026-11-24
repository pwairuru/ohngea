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
import { ChildManager } from "../components/FullCrudPage";
import type { CrudChildConfig } from "../lib/fieldConfigs";
import { Smartphone } from "lucide-react";

const deviceLines: CrudChildConfig = {
  title: "Lines",
  resourceOf: (id) => `/devices/${id}/lines`,
  idKey: "device_line_uuid",
  keyFields: [
    { name: "line_number", label: "Line", required: true },
    { name: "user_id", label: "User ID" },
    { name: "display_name", label: "Display name" },
  ],
  advancedFields: [
    { name: "server_address", label: "Server" },
    { name: "server_address_primary", label: "Server primary" },
    { name: "server_address_secondary", label: "Server secondary" },
    { name: "outbound_proxy_primary", label: "Outbound proxy 1" },
    { name: "outbound_proxy_secondary", label: "Outbound proxy 2" },
    { name: "label", label: "Label" },
    { name: "auth_id", label: "Auth ID" },
    { name: "password", label: "Password" },
    { name: "sip_port", label: "SIP port" },
    { name: "sip_transport", label: "Transport" },
    { name: "register_expires", label: "Register expires" },
    { name: "shared_line", label: "Shared line" },
    { name: "enabled", label: "Enabled", type: "checkbox" },
  ],
};

const deviceKeys: CrudChildConfig = {
  title: "Keys",
  resourceOf: (id) => `/devices/${id}/keys`,
  idKey: "device_key_uuid",
  keyFields: [
    { name: "device_key_category", label: "Category" },
    { name: "device_key_type", label: "Type" },
    { name: "device_key_value", label: "Value" },
  ],
  advancedFields: [
    { name: "device_key_id", label: "Key ID" },
    { name: "device_key_vendor", label: "Vendor" },
    { name: "device_key_subtype", label: "Subtype" },
    { name: "device_key_line", label: "Line" },
    { name: "device_key_extension", label: "Extension" },
    { name: "device_key_protected", label: "Protected", type: "checkbox" },
    { name: "device_key_label", label: "Label" },
    { name: "device_key_icon", label: "Icon" },
  ],
};

const deviceSettings: CrudChildConfig = {
  title: "Settings",
  resourceOf: (id) => `/devices/${id}/settings`,
  idKey: "device_setting_uuid",
  keyFields: [
    { name: "device_setting_name", label: "Name", required: true },
    { name: "device_setting_value", label: "Value" },
  ],
  advancedFields: [
    { name: "device_setting_category", label: "Category" },
    { name: "device_setting_subcategory", label: "Subcategory" },
    { name: "device_setting_enabled", label: "Enabled", type: "checkbox" },
    { name: "device_setting_description", label: "Description" },
  ],
};

type Device = {
  device_uuid: string;
  mac: string;
  vendor: string;
  model: string;
  label: string;
  enabled: boolean;
  [k: string]: unknown;
};

const DEV_BLANK = { mac: "00:11:22:33:44:55", vendor: "poly", model: "VVX250", label: "", device_template: "", device_location: "", device_serial_number: "", device_firmware_version: "", device_description: "", device_username: "", device_password: "", device_profile_uuid: "", device_uuid_alternate: "", enabled: true };

export function Devices() {
  const confirm = useConfirm();
  const [reloadKey, setReloadKey] = React.useState(0);
  const reload = () => setReloadKey((k) => k + 1);
  const [open, setOpen] = React.useState(false);
  const [editing, setEditing] = React.useState<Device | null>(null);
  const [form, setForm] = React.useState({ ...DEV_BLANK });
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
    setForm({ ...DEV_BLANK });
    setOpen(true);
  };
  const startEdit = (d: Device) => {
    setEditing(d);
    setForm({
      ...DEV_BLANK,
      mac: d.mac || "",
      vendor: d.vendor || "poly",
      model: d.model || "",
      label: (d.label as string) || "",
      device_template: (d.device_template as string) || "",
      device_location: (d.device_location as string) || "",
      device_serial_number: (d.device_serial_number as string) || "",
      device_firmware_version: (d.device_firmware_version as string) || "",
      device_description: (d.device_description as string) || "",
      device_username: (d.device_username as string) || "",
      device_password: "",
      device_profile_uuid: (d.device_profile_uuid as string) || "",
      device_uuid_alternate: (d.device_uuid_alternate as string) || "",
      enabled: !!d.enabled,
    });
    setOpen(true);
  };

  const saveFn = async () => {
    try {
      if (editing) {
        const patchBody: Record<string, unknown> = { mac: form.mac, enabled: form.enabled };
        for (const [key, value] of Object.entries(form)) {
          if (key === "mac" || key === "enabled") continue;
          if (key === "device_password" && value === "") continue;
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
        <div className="grid grid-cols-2 gap-2">
          <div><Label>Template</Label><Input value={form.device_template} onChange={(e) => setForm({ ...form, device_template: e.target.value })} placeholder="yealink/T46U" /></div>
          <div><Label>Profile UUID</Label><Input value={form.device_profile_uuid} onChange={(e) => setForm({ ...form, device_profile_uuid: e.target.value })} /></div>
        </div>
        <div className="grid grid-cols-2 gap-2">
          <div><Label>Location</Label><Input value={form.device_location} onChange={(e) => setForm({ ...form, device_location: e.target.value })} /></div>
          <div><Label>Serial Number</Label><Input value={form.device_serial_number} onChange={(e) => setForm({ ...form, device_serial_number: e.target.value })} /></div>
        </div>
        <div><Label>Firmware Version</Label><Input value={form.device_firmware_version} onChange={(e) => setForm({ ...form, device_firmware_version: e.target.value })} /></div>
        <div><Label>Alternate UUID</Label><Input value={form.device_uuid_alternate} onChange={(e) => setForm({ ...form, device_uuid_alternate: e.target.value })} placeholder="alternate device id" /></div>
        <div><Label>Description</Label><Input value={form.device_description} onChange={(e) => setForm({ ...form, device_description: e.target.value })} /></div>
        <div className="grid grid-cols-2 gap-2">
          <div><Label>Username</Label><Input value={form.device_username} onChange={(e) => setForm({ ...form, device_username: e.target.value })} /></div>
          <div><Label>Password {editing ? "(blank keeps)" : ""}</Label><Input type="password" value={form.device_password} onChange={(e) => setForm({ ...form, device_password: e.target.value })} /></div>
        </div>
        <div className="flex items-center gap-2"><input type="checkbox" id="dev-enabled" checked={form.enabled} onChange={(e) => setForm({ ...form, enabled: e.target.checked })} className="h-4 w-4" /><Label htmlFor="dev-enabled" className="cursor-pointer">{form.enabled ? "Enabled" : "Disabled"}</Label></div>
        {editing && (
          <div className="space-y-2">
            <ChildManager child={deviceLines} parentId={editing.device_uuid} />
            <ChildManager child={deviceKeys} parentId={editing.device_uuid} />
            <ChildManager child={deviceSettings} parentId={editing.device_uuid} />
          </div>
        )}
      </FormDialog>
    </>
  );
}
