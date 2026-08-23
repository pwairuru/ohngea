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
import { Users as UsersIcon } from "lucide-react";

export function Users() {
  const confirm = useConfirm();
  const [me, setMe] = React.useState<any>(null);
  const [reloadKey, setReloadKey] = React.useState(0);
  const reload = () => setReloadKey((k) => k + 1);
  const [open, setOpen] = React.useState(false);
  const [editing, setEditing] = React.useState<any | null>(null);
  const [form, setForm] = React.useState({ username: "", email: "", password: "", enabled: true });

  React.useEffect(() => {
    apiFetch("/auth/me").then(setMe).catch(() => {});
  }, [reloadKey]);

  const columns: Column<any>[] = [
    { key: "username", header: "Username", className: "font-mono" },
    { key: "email", header: "Email", cell: (r) => r.email || "-" },
    { key: "enabled", header: "Enabled", cell: (r) => <StatusBadge tone={r.enabled ? "ok" : "off"}>{r.enabled ? "enabled" : "disabled"}</StatusBadge> },
  ];

  // password strength: length 12 + number/lower/upper/special (defaults 12/true/true/true/false per app_config:119)
  const validatePassword = (pw: string) => {
    const errs: string[] = [];
    if (pw.length < 12) errs.push("at least 12 characters");
    if (!/\d/.test(pw)) errs.push("one number");
    if (!/[a-z]/.test(pw)) errs.push("one lowercase");
    if (!/[A-Z]/.test(pw)) errs.push("one uppercase");
    // special optional per default false, so not required unless enabled; keep check disabled by default
    // if (!/[\W_]/.test(pw)) errs.push("one special");
    return errs;
  };
  const pwErrors = React.useMemo(() => {
    if (!form.password) return [];
    return validatePassword(form.password);
  }, [form.password]);
  const isCreate = !editing;
  const passwordRequired = isCreate && !form.password;
  const isPasswordWeak = isCreate ? (passwordRequired || pwErrors.length > 0) : (form.password ? pwErrors.length > 0 : false);
  const passwordScore = form.password ? 4 - pwErrors.length : 0;

  const startCreate = () => { setEditing(null); setForm({ username: "", email: "", password: "", enabled: true }); setOpen(true); };
  const startEdit = (u: any) => { setEditing(u); setForm({ username: u.username || "", email: u.email || "", password: "", enabled: !!u.enabled }); setOpen(true); };

  const saveFn = async () => {
    if (isPasswordWeak) {
      toastError(new Error(passwordRequired ? "password required" : `weak password: ${pwErrors.join(", ")}`));
      return;
    }
    try {
      if (editing) {
        const payload: any = { username: form.username, email: form.email, enabled: form.enabled };
        if (form.password) payload.password = form.password;
        await apiFetch(`/users/${editing.user_uuid}`, { method: "PATCH", body: JSON.stringify(payload) });
        toast(`User ${form.username} updated`, { variant: "success" });
      } else {
        await apiFetch("/users", { method: "POST", body: JSON.stringify(form) });
        toast(`User ${form.username} created`, { variant: "success" });
      }
      setOpen(false); setEditing(null); reload();
    } catch (err) { toastError(err); }
  };
  const { run: save, pending: saving } = useAsyncAction(saveFn);

  const toggleEnabled = async (u: any) => {
    try { await apiFetch(`/users/${u.user_uuid}`, { method: "PATCH", body: JSON.stringify({ enabled: !u.enabled }) }); toast(`User ${u.username} ${!u.enabled ? "enabled" : "disabled"}`); reload(); } catch (err) { toastError(err); }
  };
  const deleteUser = async (u: any) => {
    const ok = await confirm({ title: `Delete user ${u.username}?`, description: "This cannot be undone.", confirmLabel: "Delete" });
    if (!ok) return;
    try { await apiFetch(`/users/${u.user_uuid}`, { method: "DELETE" }); toast(`User ${u.username} deleted`); reload(); } catch (err) { toastError(err); }
  };

  // Task 5: protected toggle persist — PUT /groups/{id}/permissions now stores permission_protected per item
  // GET supports ?category= filter via Permission.name.startswith(category+"_") and ?view=protected
  // Save payload must send protected flag: {permission_names, permission_protected, protected_map, items}
  const saveGroupPermissions = async (
    groupUuid: string,
    permsList: Array<{ name: string; permission_uuid?: string; assigned: boolean; protected?: boolean }>
  ) => {
    const permission_names = permsList.filter((p) => p.assigned).map((p) => p.name);
    const permission_uuids = permsList.filter((p) => p.assigned && p.permission_uuid).map((p) => p.permission_uuid as string);
    // protected_map from body protected flag — handles both name and uuid keys
    // legacy: permission_protected dict + protected_map dict + items with protected attr
    const permission_protected: Record<string, boolean> = {};
    const protected_map: Record<string, boolean> = {};
    permsList.forEach((p) => {
      if (p.protected) {
        permission_protected[p.name] = true;
        protected_map[p.name] = true;
        if (p.permission_uuid) protected_map[p.permission_uuid] = true;
      }
    });
    // items shape for router: {permission_name, permission_uuid, assigned, protected} — matches brief `hasattr(item,'protected')`
    const items = permsList.map((p) => ({
      permission_name: p.name,
      permission_uuid: p.permission_uuid,
      name: p.name,
      assigned: p.assigned,
      protected: !!p.protected,
    }));
    try {
      await apiFetch(`/groups/${groupUuid}/permissions`, {
        method: "PUT",
        body: JSON.stringify({ permission_names, permission_uuids, permission_protected, protected_map, items }),
      });
      toast(`Permissions saved for group ${groupUuid}`, { variant: "success" });
    } catch (err) {
      toastError(err);
    }
  };

  // Example usage: fetch with category + view=protected to verify persist
  // await apiFetch(`/groups/${groupUuid}/permissions?category=dialplan&view=protected`)
  // JWT refresh on group change: call POST /auth/refresh to get new token with updated perms

  return (
    <>
      <PageHeader title="Users & Groups" description="Users per tenant (RLS) and RBAC groups" actions={<Button onClick={startCreate}>Add User</Button>} />
      <Card><CardHeader><CardTitle>Me — JWT RLS</CardTitle></CardHeader><CardContent>{me ? <div className="font-mono text-sm">{me.username} — {me.user_uuid} — domain {me.domain_id} — perms via group_permissions</div> : <div className="text-muted-foreground text-sm">Login to see user — POST /auth/login {"{username, password, domain}"}</div>}</CardContent></Card>
      <DataTable
        resource="/users"
        columns={columns}
        rowKey={(r) => r.user_uuid}
        reloadKey={reloadKey}
        perPage={20}
        searchPlaceholder="Search users…"
        emptyTitle="No users"
        emptyDescription="Seed creates superadmin"
        emptyIcon={UsersIcon}
        emptyAction={<Button onClick={startCreate}><UsersIcon className="h-4 w-4" /> Add User</Button>}
        rowActions={(u) => (
          <div className="flex justify-end gap-1">
            <Button variant="ghost" size="sm" onClick={() => startEdit(u)}>Edit</Button>
            <Button variant="ghost" size="sm" onClick={() => toggleEnabled(u)}>{u.enabled ? "Disable" : "Enable"}</Button>
            <Button variant="ghost" size="sm" className="text-destructive hover:text-destructive" onClick={() => deleteUser(u)}>Delete</Button>
          </div>
        )}
      />
      <Card><CardHeader><CardTitle>RBAC Groups</CardTitle></CardHeader><CardContent className="text-sm text-muted-foreground">Groups: superadmin (all perms), user. Permissions: domain_view, extension_add, dialplan_view, gateway_view, etc. Stored in <code>groups</code> + <code>group_permissions</code>, checked via <code>require_permission</code> dep.</CardContent></Card>
      <FormDialog open={open} onOpenChange={(v) => { setOpen(v); if (!v) setEditing(null); }} title={editing ? "Edit User" : "Add User"} onSubmit={save} pending={saving} disabled={isPasswordWeak} submitLabel={editing ? "Save" : "Create"}>
        <div><Label>Username *</Label><Input value={form.username} onChange={(e) => setForm({ ...form, username: e.target.value })} placeholder="alice" required /></div>
        <div><Label>Email</Label><Input value={form.email} onChange={(e) => setForm({ ...form, email: e.target.value })} placeholder="alice@example.com" type="email" /></div>
        <div>
          <Label>{editing ? "New Password (leave blank to keep)" : "Password *"}</Label>
          <Input type="password" value={form.password} onChange={(e) => setForm({ ...form, password: e.target.value })} required={isCreate} placeholder={isCreate ? "min 12 chars, number, lower, upper" : ""} />
          {form.password ? (
            <div className="mt-2 space-y-1">
              <div className="flex gap-1">
                {[0,1,2,3].map((i) => (
                  <div key={i} className={`h-1 flex-1 rounded ${i < passwordScore ? (passwordScore===4 ? "bg-green-500" : passwordScore>=2 ? "bg-yellow-500" : "bg-red-500") : "bg-muted"}`} />
                ))}
              </div>
              {pwErrors.length > 0 ? (
                <p className="text-xs text-destructive">Weak: missing {pwErrors.join(", ")}</p>
              ) : (
                <p className="text-xs text-green-600">Strong password</p>
              )}
            </div>
          ) : isCreate ? (
            <p className="text-xs text-muted-foreground mt-1">Password required — min 12 chars with number, lowercase, uppercase</p>
          ) : null}
        </div>
        <div className="flex items-center gap-2"><input type="checkbox" id="user-enabled" checked={form.enabled} onChange={(e) => setForm({ ...form, enabled: e.target.checked })} className="h-4 w-4" /><Label htmlFor="user-enabled" className="cursor-pointer">{form.enabled ? "Enabled" : "Disabled"}</Label></div>
      </FormDialog>
    </>
  );
}
