import * as React from "react";
import { apiFetch } from "../lib/api";
import { FullCrudPage } from "../components/FullCrudPage";
import { CRUD_CONFIGS } from "../lib/fieldConfigs";
import { Button } from "../components/ui/button";
import { Input } from "../components/ui/input";
import { FormDialog } from "../components/FormDialog";
import { toast, toastError } from "../lib/toast";
import { useAsyncAction } from "../lib/useAsyncAction";

function PermissionsDialog({ group, onClose }: { group: any; onClose: () => void }) {
  const [perms, setPerms] = React.useState<any[]>([]);
  const [checked, setChecked] = React.useState<Record<string, boolean>>({});
  const [q, setQ] = React.useState("");

  React.useEffect(() => {
    apiFetch(`/groups/${group.group_uuid}/permissions?per_page=100`)
      .then((r: any) => {
        const data = r?.data ?? [];
        setPerms(data);
        const c: Record<string, boolean> = {};
        for (const p of data) c[p.permission_uuid] = !!p.assigned;
        setChecked(c);
      })
      .catch((err) => toastError(err));
  }, [group.group_uuid]);

  const { run: save, pending } = useAsyncAction(async () => {
    try {
      const uuids = Object.entries(checked).filter(([, v]) => v).map(([k]) => k);
      await apiFetch(`/groups/${group.group_uuid}/permissions`, {
        method: "PUT",
        body: JSON.stringify({ permission_uuids: uuids }),
      });
      toast(`Permissions saved for ${group.group_name}`, { variant: "success" });
      onClose();
    } catch (err) { toastError(err); }
  });

  const filtered = perms.filter((p) => !q || p.name.toLowerCase().includes(q.toLowerCase()));

  return (
    <FormDialog open onOpenChange={(v) => { if (!v) onClose(); }} title={`Permissions — ${group.group_name}`} onSubmit={save} pending={pending} submitLabel="Save">
      <div className="space-y-2 max-h-[60vh] overflow-auto pr-1">
        <Input value={q} onChange={(e) => setQ(e.target.value)} placeholder="Filter permissions…" />
        {filtered.slice(0, 200).map((p) => (
          <label key={p.permission_uuid} className="flex items-center gap-2 text-sm border rounded px-2 py-1">
            <input
              type="checkbox"
              checked={!!checked[p.permission_uuid]}
              onChange={(e) => setChecked({ ...checked, [p.permission_uuid]: e.target.checked })}
              className="h-4 w-4"
            />
            <span className="font-mono">{p.name}</span>
          </label>
        ))}
      </div>
    </FormDialog>
  );
}

export function Groups() {
  const [permGroup, setPermGroup] = React.useState<any | null>(null);

  const act = async (row: any, action: "copy" | "toggle", reload: () => void) => {
    try {
      const body = action === "copy" ? { group_name: `${row.group_name}-copy` } : {};
      await apiFetch(`/groups/${row.group_uuid}/${action}`, { method: "POST", body: JSON.stringify(body) });
      toast(`Group ${action} done`, { variant: "success" });
      reload();
    } catch (err) { toastError(err); }
  };

  return (
    <>
      <FullCrudPage
        config={CRUD_CONFIGS.groups}
        extraRowActions={(row: any, reload: () => void) => (
          <>
            <Button variant="ghost" size="sm" onClick={() => setPermGroup(row)}>Permissions</Button>
            <Button variant="ghost" size="sm" onClick={() => act(row, "copy", reload)}>Copy</Button>
            <Button variant="ghost" size="sm" onClick={() => act(row, "toggle", reload)}>Toggle</Button>
          </>
        )}
      />
      {permGroup && <PermissionsDialog group={permGroup} onClose={() => setPermGroup(null)} />}
    </>
  );
}
