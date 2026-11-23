import * as React from "react";
import { apiFetch } from "../lib/api";
import { PageHeader } from "../components/shell/PageHeader";
import { Button } from "../components/ui/button";
import { Label } from "../components/ui/label";
import { Input } from "../components/ui/input";
import { Card, CardContent, CardHeader, CardTitle } from "../components/ui/card";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "../components/ui/table";
import { FormDialog } from "../components/FormDialog";
import { toast, toastError } from "../lib/toast";
import { useAsyncAction } from "../lib/useAsyncAction";

export function UserSettings() {
  const [userUuid, setUserUuid] = React.useState("");
  const [username, setUsername] = React.useState("");
  const [settings, setSettings] = React.useState<any[]>([]);
  const [allSettings, setAllSettings] = React.useState<any[] | null>(null);
  const [open, setOpen] = React.useState(false);
  const [form, setForm] = React.useState({ category: "profile", subcategory: "", name: "", value: "", enabled: true, order: "", description: "" });

  const load = React.useCallback(() => {
    apiFetch("/auth/me")
      .then((me: any) => {
        setUserUuid(me?.user_uuid ?? "");
        setUsername(me?.username ?? "");
        if (me?.user_uuid) {
          apiFetch(`/users/${me.user_uuid}/settings`)
            .then((r: any) => setSettings(r?.data ?? []))
            .catch(() => setSettings([]));
        }
        // Tenant-wide list (new /user_settings endpoint; 403 without perm).
        apiFetch(`/user_settings`)
          .then((r: any) => setAllSettings(r?.data ?? []))
          .catch(() => setAllSettings(null));
      })
      .catch((err) => toastError(err));
  }, []);
  React.useEffect(() => { load(); }, [load]);

  const { run: save, pending } = useAsyncAction(async () => {
    try {
      const next = [...settings.filter((s) => s.user_setting_name !== form.name), {
        category: form.category || null,
        subcategory: form.subcategory || null,
        name: form.name,
        value: form.value,
        enabled: form.enabled,
        order: form.order === "" ? null : Number(form.order),
        description: form.description || null,
      }];
      await apiFetch(`/users/${userUuid}/settings`, { method: "PUT", body: JSON.stringify({ settings: next }) });
      toast("Setting saved", { variant: "success" });
      setOpen(false);
      setForm({ category: "profile", subcategory: "", name: "", value: "", enabled: true, order: "", description: "" });      load();
    } catch (err) { toastError(err); }
  });

  return (
    <>
      <PageHeader title="My Settings" description={`Personal preferences for ${username || "…"}`} actions={<Button onClick={() => setOpen(true)}>Add Setting</Button>} />
      <Card>
        <CardHeader><CardTitle>Preferences</CardTitle></CardHeader>
        <CardContent>
          <div className="hidden md:block">
            <Table>
              <TableHeader><TableRow><TableHead>Category</TableHead><TableHead>Name</TableHead><TableHead>Value</TableHead><TableHead>Enabled</TableHead></TableRow></TableHeader>
              <TableBody>
                {settings.length ? settings.map((s: any, i: number) => (
                  <TableRow key={s.user_setting_uuid ?? i}>
                    <TableCell className="font-mono text-xs">{s.category ?? s.user_setting_category ?? "—"}</TableCell>
                    <TableCell className="font-mono text-xs">{s.name ?? s.user_setting_name ?? "—"}</TableCell>
                    <TableCell className="font-mono text-xs">{String(s.value ?? s.user_setting_value ?? "—").slice(0, 60)}</TableCell>
                    <TableCell>{(s.enabled ?? s.user_setting_enabled ?? true) ? "yes" : "no"}</TableCell>
                  </TableRow>
                )) : <TableRow><TableCell colSpan={4} className="text-center text-muted-foreground py-8">No settings</TableCell></TableRow>}
              </TableBody>
            </Table>
          </div>
          <div className="md:hidden grid gap-3">
            {settings.map((s: any, i: number) => (
              <Card key={s.user_setting_uuid ?? i} className="p-3"><div className="font-mono text-sm">{s.name ?? s.user_setting_name}</div><div className="font-mono text-xs text-muted-foreground">{String(s.value ?? s.user_setting_value ?? "")}</div></Card>
            ))}
            {settings.length === 0 && <div className="text-center text-muted-foreground py-4">No settings</div>}
          </div>
        </CardContent>
      </Card>
      {allSettings !== null && (
        <Card className="mt-4">
          <CardHeader><CardTitle>All Tenant Settings (admin)</CardTitle></CardHeader>
          <CardContent>
            <div className="overflow-auto">
              <Table>
                <TableHeader><TableRow><TableHead>User</TableHead><TableHead>Category</TableHead><TableHead>Name</TableHead><TableHead>Value</TableHead></TableRow></TableHeader>
                <TableBody>
                  {allSettings.length ? allSettings.map((s: any, i: number) => (
                    <TableRow key={s.user_setting_uuid ?? i}>
                      <TableCell className="font-mono text-xs">{String(s.user_uuid ?? "—").slice(0, 8)}</TableCell>
                      <TableCell className="font-mono text-xs">{s.category ?? s.user_setting_category ?? "—"}</TableCell>
                      <TableCell className="font-mono text-xs">{s.name ?? s.user_setting_name ?? "—"}</TableCell>
                      <TableCell className="font-mono text-xs">{String(s.value ?? s.user_setting_value ?? "—").slice(0, 60)}</TableCell>
                    </TableRow>
                  )) : <TableRow><TableCell colSpan={4} className="text-center text-muted-foreground py-8">No tenant settings</TableCell></TableRow>}
                </TableBody>
              </Table>
            </div>
          </CardContent>
        </Card>
      )}
      <FormDialog open={open} onOpenChange={setOpen} title="Add Setting" onSubmit={save} pending={pending} submitLabel="Save">
        <div><Label>Category</Label><Input value={form.category} onChange={(e) => setForm({ ...form, category: e.target.value })} placeholder="profile" /></div>
        <div><Label>Subcategory</Label><Input value={form.subcategory} onChange={(e) => setForm({ ...form, subcategory: e.target.value })} /></div>
        <div><Label>Name *</Label><Input value={form.name} onChange={(e) => setForm({ ...form, name: e.target.value })} placeholder="language" /></div>
        <div><Label>Value</Label><Input value={form.value} onChange={(e) => setForm({ ...form, value: e.target.value })} placeholder="en" /></div>
        <div><Label>Order</Label><Input type="number" value={form.order} onChange={(e) => setForm({ ...form, order: e.target.value })} placeholder="0" /></div>
        <div><Label>Description</Label><Input value={form.description} onChange={(e) => setForm({ ...form, description: e.target.value })} placeholder="Description" /></div>
        <div className="flex items-center gap-2"><input type="checkbox" id="uset-enabled" checked={form.enabled} onChange={(e) => setForm({ ...form, enabled: e.target.checked })} className="h-4 w-4" /><Label htmlFor="uset-enabled" className="cursor-pointer">Enabled</Label></div>
      </FormDialog>
    </>
  );
}
