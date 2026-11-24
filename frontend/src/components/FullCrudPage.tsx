import * as React from "react";
import { apiFetch } from "../lib/api";
import { DataTable } from "./DataTable";
import type { Column } from "./types";
import { PageHeader } from "./shell/PageHeader";
import { Button } from "./ui/button";
import { Label } from "./ui/label";
import { Input } from "./ui/input";
import { FormDialog } from "./FormDialog";
import { toast, toastError } from "../lib/toast";
import { useAsyncAction } from "../lib/useAsyncAction";
import { Pencil, Trash2, Power, PowerOff, RotateCcw, Play } from "lucide-react";
import type { CrudConfig, CrudChildConfig, CrudField } from "../lib/fieldConfigs";

export type FormState = Record<string, string | boolean>;

export function emptyForm(fields: CrudField[]): FormState {
  const s: FormState = {};
  for (const f of fields) s[f.name] = f.type === "checkbox" ? false : "";
  return s;
}

export function formFromRow(fields: CrudField[], row: any): FormState {
  const s: FormState = {};
  for (const f of fields) {
    const v = row?.[f.name];
    s[f.name] = f.type === "checkbox" ? !!v : v === null || v === undefined ? "" : String(v);
  }
  return s;
}

export function toPayload(fields: CrudField[], form: FormState): any {
  const p: any = {};
  for (const f of fields) {
    const v = form[f.name];
    if (f.type === "checkbox") {
      p[f.name] = !!v;
      continue;
    }
    if (v === "" || v === null || v === undefined) continue;
    p[f.name] = v;
  }
  return p;
}

export function FieldInput({ field, value, onChange }: { field: CrudField; value: string | boolean; onChange: (v: string | boolean) => void }) {
  if (field.type === "checkbox") {
    return (
      <div className="flex items-center gap-2">
        <input
          type="checkbox"
          id={`fld-${field.name}`}
          checked={!!value}
          onChange={(e) => onChange(e.target.checked)}
          className="h-4 w-4"
        />
        <Label htmlFor={`fld-${field.name}`} className="cursor-pointer">
          {field.label}{field.required ? " *" : ""}
        </Label>
      </div>
    );
  }
  return (
    <div>
      <Label htmlFor={`fld-${field.name}`}>
        {field.label}{field.required ? " *" : ""}
      </Label>
      <Input
        id={`fld-${field.name}`}
        type={field.type === "number" ? "number" : field.type === "email" ? "email" : field.type === "password" ? "password" : field.type === "tel" ? "tel" : "text"}
        required={!!field.required}
        value={value as string}
        onChange={(e) => onChange(e.target.value)}
        placeholder={field.placeholder ?? field.name}
      />
    </div>
  );
}

export function ChildManager({ child, parentId }: { child: CrudChildConfig; parentId: string }) {
  const [rows, setRows] = React.useState<any[]>([]);
  const [adding, setAdding] = React.useState(false);
  const [showAdv, setShowAdv] = React.useState(false);
  const [details, setDetails] = React.useState<{ row: any; items: any[] } | null>(null);
  const [form, setForm] = React.useState<FormState>(() => emptyForm([...child.keyFields, ...child.advancedFields]));
  const resource = child.resourceOf(parentId);

  const load = React.useCallback(() => {
    apiFetch(resource)
      .then((r: any) => setRows(r?.data ?? r ?? []))
      .catch(() => setRows([]));
  }, [resource]);
  React.useEffect(() => { load(); }, [load]);

  const add = async () => {
    try {
      await apiFetch(resource, { method: "POST", body: JSON.stringify(toPayload([...child.keyFields, ...child.advancedFields], form)) });
      toast(`${child.title} added`, { variant: "success" });
      setForm(emptyForm([...child.keyFields, ...child.advancedFields]));
      setAdding(false);
      load();
    } catch (err) { toastError(err); }
  };

  const del = async (row: any) => {
    const id = row[child.idKey];
    if (!id || !confirm(`Delete ${child.title} entry?`)) return;
    try {
      await apiFetch(`${resource}/${id}`, { method: "DELETE" });
      load();
    } catch (err) { toastError(err); }
  };

  const toggleRow = async (row: any) => {
    if (!child.toggle) return;
    const id = row[child.idKey];
    const cur = String(row[child.toggle.field] ?? "");
    try {
      await apiFetch(`${resource}/${id}`, { method: "PATCH", body: JSON.stringify({ [child.toggle.field]: cur === child.toggle.on ? child.toggle.off : child.toggle.on }) });
      load();
    } catch (err) { toastError(err); }
  };

  const playRow = async (row: any) => {
    try {
      const res: any = await apiFetch(`${resource}/${row[child.idKey]}/playback`);
      const url = res?.url ?? res;
      if (typeof url === "string" && url) window.open(url, "_blank");
      else toast("No playback URL returned");
    } catch (err) { toastError(err); }
  };

  const showDetails = async (row: any) => {
    if (!child.detailsResourceOf) return;
    try {
      const res: any = await apiFetch(child.detailsResourceOf(parentId, String(row[child.idKey])));
      setDetails({ row, items: res?.data ?? [] });
    } catch (err) { toastError(err); }
  };

  return (
    <div className="border rounded p-3 space-y-2 bg-muted/20">
      <div className="flex items-center justify-between">
        <span className="text-sm font-semibold">{child.title} ({rows.length})</span>
        {!child.readOnly && <Button variant="outline" size="sm" type="button" onClick={() => setAdding((v) => !v)}>{adding ? "Cancel" : "Add"}</Button>}
      </div>
      {rows.map((r: any, i: number) => (
        <div key={r[child.idKey] ?? i} className="flex items-center justify-between gap-2 text-sm border rounded px-2 py-1 bg-card">
          <span className="truncate">
            {child.keyFields.slice(0, 3).map((f) => (r[f.name] === true ? f.label : r[f.name] === false ? "" : String(r[f.name] ?? "")).trim()).filter(Boolean).join(" · ") || "—"}
          </span>
          <span className="flex gap-1 shrink-0">
            {child.toggle && <Button variant="ghost" size="sm" type="button" onClick={() => toggleRow(r)} aria-label={child.toggle.label}>{String(r[child.toggle.field]) === child.toggle.on ? child.toggle.on : child.toggle.off}</Button>}
            {child.playback && <Button variant="ghost" size="sm" type="button" onClick={() => playRow(r)} aria-label="Play">Play</Button>}
            {child.detailsResourceOf && <Button variant="ghost" size="sm" type="button" onClick={() => showDetails(r)} aria-label={`Details`}>Details</Button>}
            {!child.readOnly && <Button variant="ghost" size="sm" type="button" className="text-destructive" onClick={() => del(r)} aria-label={`Delete ${child.title}`}>Delete</Button>}
          </span>
        </div>
      ))}
      {adding && (
        <div className="space-y-2">
          {child.keyFields.map((f) => (
            <FieldInput key={f.name} field={f} value={form[f.name]} onChange={(v) => setForm({ ...form, [f.name]: v })} />
          ))}
          {child.advancedFields.length > 0 && (
            <Button variant="outline" size="sm" className="w-full" type="button" onClick={() => setShowAdv((v) => !v)}>
              {showAdv ? "Hide Advanced" : `Show Advanced (${child.advancedFields.length} fields)`}
            </Button>
          )}
          {showAdv && child.advancedFields.map((f) => (
            <FieldInput key={f.name} field={f} value={form[f.name]} onChange={(v) => setForm({ ...form, [f.name]: v })} />
          ))}
          <Button size="sm" className="w-full" type="button" onClick={add}>Save {child.title}</Button>
        </div>
      )}
      {details && (
        <div className="border rounded p-2 space-y-1 bg-card">
          <div className="flex items-center justify-between">
            <span className="text-sm font-semibold">Session details ({details.items.length})</span>
            <Button variant="ghost" size="sm" type="button" onClick={() => setDetails(null)}>Close</Button>
          </div>
          {details.items.length === 0 && <div className="text-xs text-muted-foreground">No details</div>}
          {details.items.slice(0, 20).map((d: any, i: number) => (
            <div key={i} className="text-xs font-mono break-all border-t pt-1">
              {Object.entries(d).filter(([k]) => !k.startsWith("_")).slice(0, 6).map(([k, v]) => `${k}=${String(v)}`).join(" ")}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

export function FullCrudPage({ config, extraRowActions }: { config: CrudConfig; extraRowActions?: (row: any, reload: () => void) => React.ReactNode }) {
  const allFields = [...config.keyFields, ...config.advancedFields];
  const [open, setOpen] = React.useState(false);
  const [editing, setEditing] = React.useState<any | null>(null);
  const [form, setForm] = React.useState<FormState>(() => emptyForm(allFields));
  const [showAdvanced, setShowAdvanced] = React.useState(false);
  const [reloadKey, setReloadKey] = React.useState(0);
  const reload = () => setReloadKey((k) => k + 1);

  const enabledField = allFields.find((f) => /(^|_)enabled$/.test(f.name))?.name;

  const columns: Column<any>[] = config.keyFields.slice(0, 4).map((f) => ({
    key: f.name,
    header: f.label,
    cell: (r: any) => {
      const v = r[f.name];
      if (typeof v === "boolean") return v ? "yes" : "no";
      if (v === null || v === undefined || v === "") return "—";
      const s = String(v);
      return s.length > 60 ? s.slice(0, 60) + "…" : s;
    },
  }));

  const startCreate = () => {
    setEditing(null);
    setForm(emptyForm(allFields));
    setShowAdvanced(false);
    setOpen(true);
  };
  const startEdit = (row: any) => {
    setEditing(row);
    setForm(formFromRow(allFields, row));
    setShowAdvanced(false);
    setOpen(true);
  };
  const rowId = (row: any) => row[config.idKey];

  const saveFn = async () => {
    try {
      const payload = toPayload(allFields, form);
      if (editing) {
        await apiFetch(`${config.resource}/${rowId(editing)}`, { method: "PATCH", body: JSON.stringify(payload) });
        toast(`${config.singular} updated`, { variant: "success" });
      } else {
        await apiFetch(config.resource, { method: "POST", body: JSON.stringify(payload) });
        toast(`${config.singular} created`, { variant: "success" });
      }
      setOpen(false);
      setEditing(null);
      reload();
    } catch (err) { toastError(err); }
  };
  const { run: save, pending: saving } = useAsyncAction(saveFn);

  const toggleEnabled = async (row: any) => {
    if (!enabledField) return;
    try {
      await apiFetch(`${config.resource}/${rowId(row)}`, { method: "PATCH", body: JSON.stringify({ [enabledField]: !row[enabledField] }) });
      reload();
    } catch (err) { toastError(err); }
  };
  const del = async (row: any) => {
    if (!confirm(`Delete ${config.singular} ${row[config.keyFields[0].name] ?? rowId(row)}?`)) return;
    try {
      await apiFetch(`${config.resource}/${rowId(row)}`, { method: "DELETE" });
      toast(`${config.singular} deleted`);
      reload();
    } catch (err) { toastError(err); }
  };
  const retry = async (row: any) => {
    if (!config.retryAction) return;
    try {
      await apiFetch(`${config.resource}/${rowId(row)}${config.retryAction}`, { method: "POST" });
      toast("Retry queued", { variant: "success" });
      reload();
    } catch (err) { toastError(err); }
  };
  const customAct = async (row: any, suffix: string, label: string) => {
    try {
      const res: any = await apiFetch(`${config.resource}/${rowId(row)}${suffix}`, { method: "POST", body: JSON.stringify({}) });
      toast(`${label}: ${res?.send ? `attempted ${res.send.attempted}` : "done"}`, { variant: "success" });
      reload();
    } catch (err) { toastError(err); }
  };
  const playback = async (row: any) => {
    try {
      const res: any = await apiFetch(`${config.resource}/${rowId(row)}/presign`);
      const url = res?.url ?? res;
      if (typeof url === "string" && url) window.open(url, "_blank", "noopener");
      else toast("No playback URL returned");
    } catch (err) { toastError(err); }
  };

  return (
    <>
      <PageHeader
        title={config.title}
        description={config.description}
        actions={!config.readOnly ? <Button onClick={startCreate}>Add {config.singular}</Button> : undefined}
      />
      <DataTable
        resource={config.resource}
        columns={columns}
        rowKey={(r: any) => r[config.idKey]}
        reloadKey={reloadKey}
        perPage={20}
        searchPlaceholder={`Search ${config.title.toLowerCase()}…`}
        emptyTitle={`No ${config.title.toLowerCase()}`}
        emptyDescription={config.readOnly ? undefined : "Create via Add button"}
        rowActions={(row: any) => (
          <div className="flex justify-end gap-1">
            {!config.readOnly && (
              <Button variant="ghost" size="sm" aria-label="Edit" title="Edit" onClick={() => startEdit(row)}>
                <Pencil className="h-4 w-4" /> Edit
              </Button>
            )}
            {enabledField && !config.readOnly && (
              <Button variant="ghost" size="sm" aria-label="Toggle" title="Enable/Disable" onClick={() => toggleEnabled(row)}>
                {row[enabledField] ? <PowerOff className="h-4 w-4" /> : <Power className="h-4 w-4" />}
              </Button>
            )}
            {config.retryAction && (
              <Button variant="ghost" size="sm" aria-label="Retry" title="Retry" onClick={() => retry(row)}>
                <RotateCcw className="h-4 w-4" /> Retry
              </Button>
            )}
            {config.customActions?.map((a) => (
              <Button key={a.suffix} variant="ghost" size="sm" title={a.label} onClick={() => customAct(row, a.suffix, a.label)}>
                {a.label}
              </Button>
            ))}
            {config.playback && (
              <Button variant="ghost" size="sm" aria-label="Play" title="Playback" onClick={() => playback(row)}>
                <Play className="h-4 w-4" /> Play
              </Button>
            )}
            {extraRowActions?.(row, reload)}
            {config.allowDelete !== false && (
              <Button variant="ghost" size="sm" className="text-destructive hover:text-destructive" aria-label="Delete" title="Delete" onClick={() => del(row)}>
                <Trash2 className="h-4 w-4" /> Delete
              </Button>
            )}
          </div>
        )}
      />
      {!config.readOnly && (
        <FormDialog
          open={open}
          onOpenChange={(v) => { setOpen(v); if (!v) setEditing(null); }}
          title={editing ? `Edit ${config.singular}` : `Add ${config.singular}`}
          onSubmit={save}
          pending={saving}
          submitLabel={editing ? "Save" : "Create"}
        >
          <div className="space-y-3 max-h-[65vh] overflow-auto pr-1">
            {config.keyFields.map((f) => (
              <FieldInput key={f.name} field={f} value={form[f.name]} onChange={(v) => setForm({ ...form, [f.name]: v })} />
            ))}
            {config.advancedFields.length > 0 && (
              <Button variant="outline" className="w-full" type="button" onClick={() => setShowAdvanced((v) => !v)}>
                {showAdvanced ? "Hide Advanced" : `Show Advanced (${config.advancedFields.length} fields)`}
              </Button>
            )}
            {showAdvanced && config.advancedFields.map((f) => (
              <FieldInput key={f.name} field={f} value={form[f.name]} onChange={(v) => setForm({ ...form, [f.name]: v })} />
            ))}
            {editing && config.children?.map((c) => (
              <ChildManager key={c.title} child={c} parentId={String(rowId(editing))} />
            ))}
          </div>
        </FormDialog>
      )}
    </>
  );
}
