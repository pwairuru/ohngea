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
import { Server } from "lucide-react";

export function SipProfiles() {
  const confirm = useConfirm();
  const [reloadKey, setReloadKey] = React.useState(0);
  const reload = () => setReloadKey((k) => k + 1);
  const [open, setOpen] = React.useState(false);
  const [editing, setEditing] = React.useState<any | null>(null);
  const [form, setForm] = React.useState({ name: "internal", sip_port: 5060, sip_ip: "", enabled: true });
  const [statusMap, setStatusMap] = React.useState<Record<string, { fs_state: string; running: boolean }>>({});
  const [fsConnected, setFsConnected] = React.useState<boolean | null>(null);

  const loadStatus = React.useCallback(() => {
    apiFetch("/sip_profiles/status")
      .then((r: any) => {
        setFsConnected(r.connected);
        const m: Record<string, { fs_state: string; running: boolean }> = {};
        (r.profiles || []).forEach((p: any) => {
          m[p.name] = { fs_state: p.fs_state, running: p.running };
          m[p.sip_profile_uuid] = { fs_state: p.fs_state, running: p.running };
        });
        setStatusMap(m);
      })
      .catch(() => setFsConnected(false));
  }, []);
  React.useEffect(() => { loadStatus(); const id = setInterval(loadStatus, 30000); return () => clearInterval(id); }, [loadStatus]);

  const columns: Column<any>[] = [
    { key: "name", header: "Name", className: "font-mono" },
    { key: "sip_port", header: "Port", className: "num" },
    { key: "enabled", header: "Enabled", cell: (r: any) => <StatusBadge tone={r.enabled ? "ok" : "off"}>{r.enabled ? "enabled" : "disabled"}</StatusBadge> },
    { key: "fs_status", header: "FS Status", cell: (r: any) => {
        const st = statusMap[r.name] || statusMap[r.sip_profile_uuid];
        const fsState = st?.fs_state || (fsConnected === false ? "UNKNOWN" : "—");
        const running = st?.running;
        let dot = "bg-gray-300";
        let textColor = "text-muted-foreground";
        if (fsConnected === false) { dot = "bg-gray-300"; textColor = "text-muted-foreground"; }
        else if (running === true) { dot = "bg-green-500"; textColor = "text-green-700"; }
        else if (running === false && st) { dot = "bg-red-500"; textColor = "text-red-700"; }
        return <span data-testid={`sip-status-${r.name}`} className={`inline-flex items-center gap-1.5 text-xs ${textColor}`}><span className={`h-2 w-2 rounded-full ${dot} ${running ? "shadow-[0_0_4px_rgba(34,197,94,0.5)]" : ""}`} />{fsState}</span>;
      }
    },
  ];

  const startCreate = () => { setEditing(null); setForm({ name: "internal", sip_port: 5060, sip_ip: "", enabled: true }); setOpen(true); };
  const startEdit = (p: any) => { setEditing(p); setForm({ name: p.name || "", sip_port: p.sip_port || 5060, sip_ip: p.sip_ip || "", enabled: !!p.enabled }); setOpen(true); };

  const saveFn = async () => {
    try {
      if (editing) await apiFetch(`/sip_profiles/${editing.sip_profile_uuid}`, { method: "PATCH", body: JSON.stringify(form) });
      else await apiFetch("/sip_profiles", { method: "POST", body: JSON.stringify(form) });
      toast(`SIP profile ${form.name} ${editing ? "updated" : "created"}`, { variant: "success" });
      setOpen(false); setEditing(null); reload(); loadStatus();
    } catch (err) { toastError(err); }
  };
  const { run: save, pending: saving } = useAsyncAction(saveFn);

  const toggleEnabled = async (p: any) => {
    try { await apiFetch(`/sip_profiles/${p.sip_profile_uuid}`, { method: "PATCH", body: JSON.stringify({ enabled: !p.enabled }) }); toast(`SIP profile ${p.name} ${!p.enabled ? "enabled" : "disabled"}`); reload(); } catch (err) { toastError(err); }
  };
  const deleteProfile = async (p: any) => {
    const ok = await confirm({ title: `Delete SIP profile ${p.name}?`, description: "This cannot be undone.", confirmLabel: "Delete" });
    if (!ok) return;
    try { await apiFetch(`/sip_profiles/${p.sip_profile_uuid}`, { method: "DELETE" }); toast(`SIP profile ${p.name} deleted`); reload(); } catch (err) { toastError(err); }
  };

  return (
    <>
      <PageHeader
        title="SIP Profiles"
        description="Sofia SIP profiles — internal/external — per domain"
        actions={
          <>
            <span data-testid="sip-fs-connected" className={`text-xs px-2 py-1 rounded-full border ${fsConnected === true ? "bg-green-50 text-green-700 border-green-200" : fsConnected === false ? "bg-red-50 text-red-700 border-red-200" : "bg-muted text-muted-foreground"}`}>
              {fsConnected === true ? "FS Connected" : fsConnected === false ? "FS Disconnected" : "Checking…"}
            </span>
            <Button onClick={startCreate}>Add Profile</Button>
          </>
        }
      />
      <DataTable
        resource="/sip_profiles"
        columns={columns}
        rowKey={(r) => r.sip_profile_uuid}
        reloadKey={reloadKey}
        perPage={20}
        searchPlaceholder="Search profiles…"
        emptyTitle="No profiles"
        emptyDescription="Create your first SIP profile"
        emptyIcon={Server}
        emptyAction={<Button onClick={startCreate}><Server className="h-4 w-4" /> Add Profile</Button>}
        rowActions={(p) => (
          <div className="flex justify-end gap-1">
            <Button variant="ghost" size="sm" onClick={() => startEdit(p)}>Edit</Button>
            <Button variant="ghost" size="sm" onClick={() => toggleEnabled(p)}>{p.enabled ? "Disable" : "Enable"}</Button>
            <Button variant="ghost" size="sm" className="text-destructive hover:text-destructive" onClick={() => deleteProfile(p)}>Delete</Button>
          </div>
        )}
      />
      <FormDialog open={open} onOpenChange={(v) => { setOpen(v); if (!v) setEditing(null); }} title={editing ? "Edit SIP Profile" : "Add SIP Profile"} onSubmit={save} pending={saving} submitLabel={editing ? "Save" : "Create"}>
        <div><Label>Name *</Label><Input value={form.name} onChange={(e) => setForm({ ...form, name: e.target.value })} /></div>
        <div><Label>Port</Label><Input type="number" value={form.sip_port} onChange={(e) => setForm({ ...form, sip_port: Number(e.target.value) })} /></div>
        <div><Label>SIP IP</Label><Input value={form.sip_ip} onChange={(e) => setForm({ ...form, sip_ip: e.target.value })} placeholder="auto" /></div>
        <div className="flex items-center gap-2"><input type="checkbox" id="sip-enabled" checked={form.enabled} onChange={(e) => setForm({ ...form, enabled: e.target.checked })} className="h-4 w-4" /><Label htmlFor="sip-enabled" className="cursor-pointer">{form.enabled ? "Enabled" : "Disabled"}</Label></div>
      </FormDialog>
    </>
  );
}
