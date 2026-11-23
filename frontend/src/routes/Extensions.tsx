import * as React from "react";
import { apiFetch } from "../lib/api";
import { DataTable } from "../components/DataTable";
import type { Column } from "../components/types";
import { PageHeader } from "../components/shell/PageHeader";
import { StatusBadge } from "../components/StatusBadge";
import { Button } from "../components/ui/button";
import { Input } from "../components/ui/input";
import { Label } from "../components/ui/label";
import { Dialog, DialogHeader, DialogTitle } from "../components/ui/dialog";
import { useConfirm } from "../components/ui/confirm";
import { toast, toastError } from "../lib/toast";
import { Phone as PhonePlus, FileUp } from "lucide-react";

type Ext = {
  extension_uuid: string;
  extension: string;
  effective_caller_id_name: string;
  effective_caller_id_number?: string;
  enabled: boolean;
  [k: string]: unknown;
};

const TEXT_FIELDS: { key: string; label: string; placeholder?: string }[] = [
  { key: "number_alias", label: "Number Alias" },
  { key: "accountcode", label: "Account Code" },
  { key: "effective_caller_id_number", label: "Caller ID Number", placeholder: "+15551234567" },
  { key: "outbound_caller_id_name", label: "Outbound CID Name" },
  { key: "outbound_caller_id_number", label: "Outbound CID Number" },
  { key: "emergency_caller_id_name", label: "Emergency CID Name" },
  { key: "emergency_caller_id_number", label: "Emergency CID Number" },
  { key: "directory_first_name", label: "Directory First Name" },
  { key: "directory_last_name", label: "Directory Last Name" },
  { key: "max_registrations", label: "Max Registrations" },
  { key: "limit_max", label: "Limit Max" },
  { key: "limit_destination", label: "Limit Destination" },
  { key: "missed_call_app", label: "Missed Call App" },
  { key: "missed_call_data", label: "Missed Call Data" },
  { key: "user_context", label: "User Context" },
  { key: "toll_allow", label: "Toll Allow" },
  { key: "call_group", label: "Call Group" },
  { key: "user_record", label: "User Record" },
  { key: "hold_music", label: "Hold Music" },
  { key: "auth_acl", label: "Auth ACL" },
  { key: "cidr", label: "CIDR" },
  { key: "sip_force_contact", label: "SIP Force Contact" },
  { key: "nibble_account", label: "Nibble Account" },
  { key: "mwi_account", label: "MWI Account" },
  { key: "sip_bypass_media", label: "SIP Bypass Media" },
  { key: "absolute_codec_string", label: "Codec String" },
  { key: "dial_string", label: "Dial String" },
  { key: "forward_all_destination", label: "Forward All →" },
  { key: "forward_busy_destination", label: "Forward Busy →" },
  { key: "forward_no_answer_destination", label: "Forward No Answer →" },
  { key: "forward_user_not_registered_destination", label: "Forward Not Registered →" },
  { key: "extension_language", label: "Language" },
  { key: "extension_dialect", label: "Dialect" },
  { key: "extension_voice", label: "Voice" },
  { key: "extension_type", label: "Type" },
  { key: "description", label: "Description" },
];

const INT_FIELDS: { key: string; label: string }[] = [
  { key: "call_timeout", label: "Call Timeout" },
  { key: "sip_force_expires", label: "SIP Force Expires" },
];

const BOOL_FIELDS: { key: string; label: string }[] = [
  { key: "directory_visible", label: "Directory Visible" },
  { key: "directory_exten_visible", label: "Directory Exten Visible" },
  { key: "call_screen_enabled", label: "Call Screen" },
  { key: "force_ping", label: "Force Ping" },
  { key: "do_not_disturb", label: "Do Not Disturb" },
  { key: "forward_all_enabled", label: "Forward All" },
  { key: "forward_busy_enabled", label: "Forward Busy" },
  { key: "forward_no_answer_enabled", label: "Forward No Answer" },
  { key: "forward_user_not_registered_enabled", label: "Forward Not Registered" },
  { key: "follow_me_enabled", label: "Follow Me" },
  { key: "enabled", label: "Enabled" },
];

const blank: Record<string, string | number | boolean> = {
  extension: "",
  password: "",
  effective_caller_id_name: "",
  voicemail_password: "",
  voicemail_mail_to: "",
  voicemail_enabled: true,
  voicemail_file: "",
  voicemail_local_after_email: false,
  voicemail_transcription_enabled: false,
  voicemail_transcription_prompt: "",
  voicemail_transcription_prompt_enabled: false,
};
for (const f of TEXT_FIELDS) blank[f.key] = "";
for (const f of INT_FIELDS) blank[f.key] = "";
for (const f of BOOL_FIELDS) blank[f.key] = f.key === "enabled";

export function Extensions() {
  const confirm = useConfirm();
  const [reloadKey, setReloadKey] = React.useState(0);
  const reload = () => setReloadKey((k) => k + 1);
  const [open, setOpen] = React.useState(false);
  const [editing, setEditing] = React.useState<Ext | null>(null);
  const [form, setForm] = React.useState({ ...blank });
  const [saving, setSaving] = React.useState(false);
  const [csvOpen, setCsvOpen] = React.useState(false);
  const [csvText, setCsvText] = React.useState("extension,name\n1001,Alice\n1002,Bob");
  const [rangeOpen, setRangeOpen] = React.useState(false);
  const [rangeStart, setRangeStart] = React.useState("");
  const [rangeCount, setRangeCount] = React.useState("5");

  const columns: Column<Ext>[] = [
    { key: "extension", header: "Extension", sortable: true, className: "num font-medium" },
    { key: "effective_caller_id_name", header: "Name", sortable: true },
    {
      key: "enabled",
      header: "Status",
      cell: (r) => (
        <button onClick={() => toggle(r)} title="Toggle enabled" className="inline-flex">
          <StatusBadge tone={r.enabled ? "ok" : "off"}>{r.enabled ? "Enabled" : "Disabled"}</StatusBadge>
        </button>
      ),
    },
  ];

  const fillFrom = (r: Ext | null) => {
    const next = { ...blank };
    if (r) {
      for (const k of Object.keys(next)) {
        const v = r[k];
        if (v !== undefined && v !== null) next[k] = v as string | number | boolean;
      }
      next.extension = r.extension ?? "";
      next.effective_caller_id_name = (r.effective_caller_id_name as string) ?? "";
      next.enabled = !!r.enabled;
      next.password = "";
    }
    return next;
  };

  const openCreate = () => { setEditing(null); setForm({ ...blank }); setOpen(true); };
  const openEdit = (r: Ext) => {
    setEditing(r);
    setForm(fillFrom(r));
    setOpen(true);
  };

  const save = async () => {
    setSaving(true);
    try {
      const payload: Record<string, unknown> = {};
      for (const [key, value] of Object.entries(form)) {
        if (key === "password" && editing && value === "") continue;
        if (value === "") continue;
        payload[key] = value;
      }
      if (editing) {
        await apiFetch(`/extensions/${editing.extension_uuid}`, {
          method: "PATCH",
          body: JSON.stringify(payload),
        });
        toast(`Extension ${form.extension} updated`, { variant: "success" });
      } else {
        const me = await apiFetch("/auth/me");
        if (!me.domain_id) throw new Error("No domain found — create a domain first");
        await apiFetch("/extensions", { method: "POST", body: JSON.stringify({ ...payload, domain_id: me.domain_id }) });
        toast(`Extension ${form.extension} created`, { variant: "success" });
      }
      setOpen(false);
      reload();
    } catch (err) {
      toastError(err);
    } finally {
      setSaving(false);
    }
  };

  const toggle = async (r: Ext) => {
    try {
      await apiFetch(`/extensions/${r.extension_uuid}`, { method: "PATCH", body: JSON.stringify({ enabled: !r.enabled }) });
      reload();
    } catch (err) {
      toastError(err);
    }
  };

  const remove = async (r: Ext) => {
    const ok = await confirm({ title: `Delete extension ${r.extension}?`, description: "This cannot be undone.", confirmLabel: "Delete" });
    if (!ok) return;
    try {
      await apiFetch(`/extensions/${r.extension_uuid}`, { method: "DELETE" });
      toast(`Extension ${r.extension} deleted`);
      reload();
    } catch (err) {
      toastError(err);
    }
  };

  const importCsv = async () => {
    try {
      const lines = csvText.split(/\r?\n/).map((l) => l.trim()).filter(Boolean);
      if (!lines.length) {
        toast("CSV is empty", { variant: "error" });
        return;
      }
      const parseLine = (line: string): string[] => {
        const out: string[] = [];
        let cur = "", inQ = false;
        for (let i = 0; i < line.length; i++) {
          const c = line[i];
          if (c === '"' && line[i + 1] === '"') { cur += '"'; i++; continue; }
          if (c === '"') { inQ = !inQ; continue; }
          if (c === "," && !inQ) { out.push(cur.trim()); cur = ""; continue; }
          cur += c;
        }
        out.push(cur.trim());
        return out;
      };
      const header = parseLine(lines[0]).map((h) => h.toLowerCase());
      const hasHeader = header.includes("extension");
      const me = await apiFetch("/auth/me");
      const payload = lines.slice(hasHeader ? 1 : 0).map((l) => {
        const v = parseLine(l);
        const ext = (v[0] || "").trim();
        // PHP rule (extension_edit.php): non-empty only — no digit regex.
        if (!ext) throw new Error(`blank extension in row: ${l}`);
        return { extension: ext, effective_caller_id_name: (v[1] || "").trim(), domain_id: me.domain_id, enabled: true };
      });
      if (!payload.length) {
        toast("No rows to import", { variant: "error" });
        return;
      }
      if (payload.length > 500) {
        toast("Max 500 rows per import", { variant: "error" });
        return;
      }
      const res = await apiFetch("/extensions/import", { method: "POST", body: JSON.stringify({ extensions: payload }) });
      const errs = res?.errors ?? [];
      toast(`${res?.imported ?? 0} extensions imported${errs.length ? `, ${errs.length} errors` : ""}`, { variant: errs.length ? "error" : "success" });
      setCsvOpen(false);
      reload();
    } catch (err) {
      toastError(err);
    }
  };

  const copyExtension = async (r: Ext) => {
    const next = window.prompt(`Copy extension ${r.extension} to new number:`, `${r.extension}`);
    if (!next || next.trim() === "" || next.trim() === r.extension) return;
    try {
      await apiFetch(`/extensions/${r.extension_uuid}/copy`, { method: "POST", body: JSON.stringify({ extension: next.trim() }) });
      toast(`Extension copied to ${next.trim()}`, { variant: "success" });
      reload();
    } catch (err) {
      toastError(err);
    }
  };

  const createRange = async () => {
    const start = rangeStart.trim();
    const count = Math.min(1000, Math.max(1, Number(rangeCount) || 0));
    if (!start) {
      toast("Start extension required", { variant: "error" });
      return;
    }
    try {
      const res = await apiFetch("/extensions:batch", { method: "POST", body: JSON.stringify({ start, range: count }) });
      toast(`${res?.created ?? 0} extensions created${res?.skipped ? `, ${res.skipped} skipped` : ""}`, { variant: "success" });
      setRangeOpen(false);
      reload();
    } catch (err) {
      toastError(err);
    }
  };

  const set = (key: string, value: string | number | boolean) => setForm((f) => ({ ...f, [key]: value }));

  return (
    <>
      <PageHeader
        title="Extensions"
        description="Internal numbers users register SIP devices against"
        actions={
          <>
            <Button variant="outline" onClick={() => setCsvOpen(true)}><FileUp className="h-4 w-4" /> CSV Import</Button>
            <Button variant="outline" onClick={() => setRangeOpen(true)}>Add Range</Button>
            <Button onClick={openCreate} data-testid="add-extension"><PhonePlus className="h-4 w-4" /> Add Extension</Button>
          </>
        }
      />
      <DataTable<Ext>
        resource="/extensions"
        columns={columns}
        rowKey={(r) => r.extension_uuid}
        reloadKey={reloadKey}
        perPage={20}
        searchPlaceholder="Search extensions…"
        emptyTitle="No extensions yet"
        emptyDescription="Create your first internal extension"
        emptyIcon={PhonePlus}
        emptyAction={<Button onClick={openCreate}><PhonePlus className="h-4 w-4" /> Add Extension</Button>}
        rowActions={(r) => (
          <div className="flex justify-end gap-1">
            <Button variant="ghost" size="sm" onClick={() => toggle(r)}>
              {r.enabled ? "Disable" : "Enable"}
            </Button>
            <Button variant="ghost" size="sm" onClick={() => openEdit(r)}>Edit</Button>
            <Button variant="ghost" size="sm" onClick={() => copyExtension(r)}>Copy</Button>
            <Button variant="ghost" size="sm" className="text-destructive hover:text-destructive" onClick={() => remove(r)}>Delete</Button>
          </div>
        )}
      />

      <Dialog open={open} onOpenChange={setOpen}>
        <DialogHeader><DialogTitle>{editing ? "Edit Extension" : "Add Extension"}</DialogTitle></DialogHeader>
        <div className="max-h-[70vh] overflow-y-auto space-y-5 pr-1">
          <section className="space-y-2">
            <h3 className="text-sm font-semibold">Account</h3>
            <div className="grid grid-cols-2 gap-2">
              <div><Label>Extension *</Label><Input value={String(form.extension)} onChange={(e) => set("extension", e.target.value)} placeholder="1001" /></div>
              <div><Label>Password {editing ? "(blank keeps)" : "*"}</Label><Input type="password" value={String(form.password)} onChange={(e) => set("password", e.target.value)} /></div>
            </div>
            <div><Label>Display Name *</Label><Input value={String(form.effective_caller_id_name)} onChange={(e) => set("effective_caller_id_name", e.target.value)} placeholder="Alice Smith" /></div>
          </section>

          <section className="space-y-2">
            <h3 className="text-sm font-semibold">Caller ID</h3>
            <div className="grid grid-cols-2 gap-2">
              {TEXT_FIELDS.slice(2, 8).map((f) => (
                <div key={f.key}><Label>{f.label}</Label><Input value={String(form[f.key] ?? "")} onChange={(e) => set(f.key, e.target.value)} placeholder={f.placeholder} /></div>
              ))}
            </div>
          </section>

          <section className="space-y-2">
            <h3 className="text-sm font-semibold">Directory & Limits</h3>
            <div className="grid grid-cols-2 gap-2">
              {TEXT_FIELDS.slice(8, 16).map((f) => (
                <div key={f.key}><Label>{f.label}</Label><Input value={String(form[f.key] ?? "")} onChange={(e) => set(f.key, e.target.value)} /></div>
              ))}
            </div>
          </section>

          <section className="space-y-2">
            <h3 className="text-sm font-semibold">Behavior</h3>
            <div className="grid grid-cols-2 gap-2">
              {TEXT_FIELDS.slice(16, 22).map((f) => (
                <div key={f.key}><Label>{f.label}</Label><Input value={String(form[f.key] ?? "")} onChange={(e) => set(f.key, e.target.value)} /></div>
              ))}
              {INT_FIELDS.map((f) => (
                <div key={f.key}><Label>{f.label}</Label><Input type="number" value={String(form[f.key] ?? "")} onChange={(e) => set(f.key, e.target.value === "" ? "" : Number(e.target.value))} /></div>
              ))}
            </div>
          </section>

          <section className="space-y-2">
            <h3 className="text-sm font-semibold">Forwarding & DND</h3>
            <div className="grid grid-cols-2 gap-2">
              {TEXT_FIELDS.slice(28, 32).map((f) => (
                <div key={f.key}><Label>{f.label}</Label><Input value={String(form[f.key] ?? "")} onChange={(e) => set(f.key, e.target.value)} /></div>
              ))}
            </div>
            <div className="flex flex-wrap gap-3">
              {BOOL_FIELDS.filter((f) => f.key.startsWith("forward_") || f.key === "do_not_disturb" || f.key === "follow_me_enabled").map((f) => (
                <label key={f.key} className="flex items-center gap-1 text-xs">
                  <input type="checkbox" checked={!!form[f.key]} onChange={(e) => set(f.key, e.target.checked)} /> {f.label}
                </label>
              ))}
            </div>
          </section>

          <section className="space-y-2">
            <h3 className="text-sm font-semibold">Voicemail</h3>
            <div className="grid grid-cols-2 gap-2">
              <div><Label>Voicemail Password</Label><Input type="password" value={String(form.voicemail_password ?? "")} onChange={(e) => set("voicemail_password", e.target.value)} /></div>
              <div><Label>Voicemail Email To</Label><Input value={String(form.voicemail_mail_to ?? "")} onChange={(e) => set("voicemail_mail_to", e.target.value)} /></div>
              <div><Label>Voicemail File</Label><Input value={String(form.voicemail_file ?? "")} onChange={(e) => set("voicemail_file", e.target.value)} /></div>
              <div><Label>Transcription Prompt</Label><Input value={String(form.voicemail_transcription_prompt ?? "")} onChange={(e) => set("voicemail_transcription_prompt", e.target.value)} /></div>
            </div>
            <div className="flex flex-wrap gap-3">
              <label className="flex items-center gap-1 text-xs">
                <input type="checkbox" checked={!!form.voicemail_enabled} onChange={(e) => set("voicemail_enabled", e.target.checked)} /> Voicemail Enabled
              </label>
              <label className="flex items-center gap-1 text-xs">
                <input type="checkbox" checked={!!form.voicemail_local_after_email} onChange={(e) => set("voicemail_local_after_email", e.target.checked)} /> Keep Local After Email
              </label>
              <label className="flex items-center gap-1 text-xs">
                <input type="checkbox" checked={!!form.voicemail_transcription_enabled} onChange={(e) => set("voicemail_transcription_enabled", e.target.checked)} /> Transcription Enabled
              </label>
              <label className="flex items-center gap-1 text-xs">
                <input type="checkbox" checked={!!form.voicemail_transcription_prompt_enabled} onChange={(e) => set("voicemail_transcription_prompt_enabled", e.target.checked)} /> Transcription Prompt Enabled
              </label>
            </div>
          </section>

          <section className="space-y-2">
            <h3 className="text-sm font-semibold">Advanced SIP</h3>
            <div className="grid grid-cols-2 gap-2">
              {[...TEXT_FIELDS.slice(22, 28), ...TEXT_FIELDS.slice(32)].map((f) => (
                <div key={f.key}><Label>{f.label}</Label><Input value={String(form[f.key] ?? "")} onChange={(e) => set(f.key, e.target.value)} /></div>
              ))}
            </div>
            <div className="flex flex-wrap gap-3">
              {BOOL_FIELDS.filter((f) => ["directory_visible", "directory_exten_visible", "call_screen_enabled", "force_ping", "enabled"].includes(f.key)).map((f) => (
                <label key={f.key} className="flex items-center gap-1 text-xs">
                  <input type="checkbox" checked={!!form[f.key]} onChange={(e) => set(f.key, e.target.checked)} /> {f.label}
                </label>
              ))}
            </div>
          </section>
        </div>
        <div className="flex gap-2 pt-3">
          <Button variant="outline" onClick={() => setOpen(false)} className="flex-1">Cancel</Button>
          <Button onClick={save} disabled={saving} className="flex-1">{editing ? "Save" : "Create"}</Button>
        </div>
      </Dialog>

      <Dialog open={rangeOpen} onOpenChange={setRangeOpen}>
        <DialogHeader><DialogTitle>Add Extension Range</DialogTitle></DialogHeader>
        <div className="space-y-3">
          <div className="text-sm text-muted-foreground">Sequential numbers from start (existing numbers are skipped, domain cap clamps the run).</div>
          <div><Label>Start Extension *</Label><Input value={rangeStart} onChange={(e) => setRangeStart(e.target.value)} placeholder="1001" /></div>
          <div><Label>Count *</Label><Input type="number" value={rangeCount} onChange={(e) => setRangeCount(e.target.value)} placeholder="5" /></div>
          <div className="flex gap-2"><Button variant="outline" onClick={() => setRangeOpen(false)} className="flex-1">Cancel</Button><Button onClick={createRange} className="flex-1">Create Range</Button></div>
        </div>
      </Dialog>

      <Dialog open={csvOpen} onOpenChange={setCsvOpen}>
        <DialogHeader><DialogTitle>CSV Import Wizard</DialogTitle></DialogHeader>
        <div className="space-y-3">
          <div className="text-sm text-muted-foreground">Paste CSV with headers extension,name — preview below.</div>
          <textarea value={csvText} onChange={(e) => setCsvText(e.target.value)} rows={6} className="w-full border rounded p-2 font-mono text-sm" />
          <div className="rounded border p-2 bg-muted/30 max-h-32 overflow-auto text-xs">
            <div className="font-medium mb-1">Preview (3 rows)</div>
            {csvText.split("\n").slice(0, 4).map((l, i) => <div key={i} className={i === 0 ? "font-bold" : ""}>{l}</div>)}
          </div>
          <div className="flex gap-2"><Button variant="outline" onClick={() => setCsvOpen(false)} className="flex-1">Cancel</Button><Button onClick={importCsv} className="flex-1">Import</Button></div>
        </div>
      </Dialog>
    </>
  );
}
