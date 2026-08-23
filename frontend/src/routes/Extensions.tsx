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
};

const blank = { extension: "", effective_caller_id_name: "", effective_caller_id_number: "", enabled: true };

export function Extensions() {
  const confirm = useConfirm();
  const [reloadKey, setReloadKey] = React.useState(0);
  const reload = () => setReloadKey((k) => k + 1);
  const [open, setOpen] = React.useState(false);
  const [step, setStep] = React.useState(1);
  const [editing, setEditing] = React.useState<Ext | null>(null);
  const [form, setForm] = React.useState(blank);
  const [saving, setSaving] = React.useState(false);
  const [csvOpen, setCsvOpen] = React.useState(false);
  const [csvText, setCsvText] = React.useState("extension,name\n1001,Alice\n1002,Bob");

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

  const openCreate = () => { setEditing(null); setForm(blank); setStep(1); setOpen(true); };
  const openEdit = (r: Ext) => {
    setEditing(r);
    setForm({
      extension: r.extension ?? "",
      effective_caller_id_name: r.effective_caller_id_name ?? "",
      effective_caller_id_number: r.effective_caller_id_number ?? "",
      enabled: !!r.enabled,
    });
    setStep(1);
    setOpen(true);
  };

  const save = async () => {
    setSaving(true);
    try {
      if (editing) {
        const patchBody: Record<string, unknown> = { extension: form.extension, enabled: form.enabled };
        for (const [key, value] of Object.entries(form)) {
          if (key === "extension" || key === "enabled") continue;
          if (value !== "") patchBody[key] = value;
        }
        await apiFetch(`/extensions/${editing.extension_uuid}`, {
          method: "PATCH",
          body: JSON.stringify(patchBody),
        });
        toast(`Extension ${form.extension} updated`, { variant: "success" });
      } else {
        const me = await apiFetch("/auth/me");
        if (!me.domain_id) throw new Error("No domain found — create a domain first");
        await apiFetch("/extensions", { method: "POST", body: JSON.stringify({ ...form, domain_id: me.domain_id }) });
        toast(`Extension ${form.extension} created`, { variant: "success" });
      }
      setOpen(false);
      setStep(1);
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
      const lines = csvText.split("\n").filter(Boolean);
      const me = await apiFetch("/auth/me");
      const payload = lines.slice(1).map((l) => {
        const v = l.split(",");
        return { extension: v[0], effective_caller_id_name: v[1], domain_id: me.domain_id, enabled: true };
      });
      await apiFetch("/extensions:batch", { method: "POST", body: JSON.stringify(payload) });
      toast(`${payload.length} extensions imported`, { variant: "success" });
      setCsvOpen(false);
      reload();
    } catch (err) {
      toastError(err);
    }
  };

  return (
    <>
      <PageHeader
        title="Extensions"
        description="Internal numbers users register SIP devices against"
        actions={
          <>
            <Button variant="outline" onClick={() => setCsvOpen(true)}><FileUp className="h-4 w-4" /> CSV Import</Button>
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
            <Button variant="ghost" size="sm" className="text-destructive hover:text-destructive" onClick={() => remove(r)}>Delete</Button>
          </div>
        )}
      />

      <Dialog open={open} onOpenChange={(v) => { setOpen(v); if (!v) setStep(1); }}>
        <DialogHeader><DialogTitle>{editing ? "Edit Extension" : "Add Extension"} — Step {step} of 3</DialogTitle></DialogHeader>
        <div className="flex gap-1 mb-4">
          {[1, 2, 3].map((s) => <div key={s} className={`h-1 flex-1 rounded ${s <= step ? "bg-primary" : "bg-muted"}`} />)}
        </div>
        {step === 1 && (
          <div className="space-y-3">
            <div><Label>Extension *</Label><Input value={form.extension} onChange={(e) => setForm({ ...form, extension: e.target.value })} placeholder="1001" /></div>
            <div><Label>Display Name *</Label><Input value={form.effective_caller_id_name} onChange={(e) => setForm({ ...form, effective_caller_id_name: e.target.value })} placeholder="Alice Smith" /></div>
            <Button onClick={() => setStep(2)} className="w-full">Next: Caller ID</Button>
          </div>
        )}
        {step === 2 && (
          <div className="space-y-3">
            <div><Label>Caller ID Number</Label><Input value={form.effective_caller_id_number} onChange={(e) => setForm({ ...form, effective_caller_id_number: e.target.value })} placeholder="+15551234567" /></div>
            <div className="flex gap-2"><Button variant="outline" onClick={() => setStep(1)} className="flex-1">Back</Button><Button onClick={() => setStep(3)} className="flex-1">Next: Review</Button></div>
          </div>
        )}
        {step === 3 && (
          <div className="space-y-3">
            <div className="text-sm space-y-1"><div>Extension: <span className="font-mono">{form.extension}</span></div><div>Name: {form.effective_caller_id_name}</div><div>Number: {form.effective_caller_id_number || "-"}</div></div>
            <div className="flex gap-2"><Button variant="outline" onClick={() => setStep(2)} className="flex-1">Back</Button><Button onClick={save} disabled={saving} className="flex-1">{editing ? "Save" : "Create"}</Button></div>
          </div>
        )}
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
