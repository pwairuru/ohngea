import * as React from "react";
import { apiFetch } from "../lib/api";
import { Card, CardContent, CardHeader, CardTitle } from "../components/ui/card";
import { Button } from "../components/ui/button";
import { Input } from "../components/ui/input";
import { Label } from "../components/ui/label";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "../components/ui/table";
import { Dialog, DialogHeader, DialogTitle } from "../components/ui/dialog";
import { Badge } from "../components/ui/badge";

export function Users() {
  const [me, setMe] = React.useState<any>(null);
  const [users, setUsers] = React.useState<any[]>([]);
  const [open, setOpen] = React.useState(false);
  const [form, setForm] = React.useState({ username: "", email: "", password: "" });

  const load = () => {
    apiFetch("/auth/me").then(setMe).catch(() => {});
    apiFetch("/users").then((r) => setUsers(r.data || [])).catch(() => {});
  };
  React.useEffect(() => { load(); }, []);

  const create = async () => {
    try { await apiFetch("/users", { method: "POST", body: JSON.stringify(form) }); setOpen(false); load(); } catch (e: any) { alert(e.message); }
  };

  return (
    <div className="p-4 space-y-4">
      <div className="flex justify-between items-center"><h1 className="text-2xl font-bold">Users & Groups</h1><Button onClick={() => setOpen(true)}>Add User</Button></div>
      <Card><CardHeader><CardTitle>Me — JWT RLS</CardTitle></CardHeader><CardContent>{me ? <div className="font-mono text-sm">{me.username} — {me.user_uuid} — domain {me.domain_id} — perms via group_permissions</div> : <div className="text-muted-foreground text-sm">Login to see user — POST /auth/login {"{username, password, domain}"}</div>}</CardContent></Card>
      <Card>
        <CardHeader><CardTitle>Users per tenant (RLS `domain_id = current_setting`)</CardTitle></CardHeader>
        <CardContent>
          <div className="hidden md:block">
            <Table>
              <TableHeader><TableRow><TableHead>Username</TableHead><TableHead>Email</TableHead><TableHead>Enabled</TableHead></TableRow></TableHeader>
              <TableBody>
                {users.map((u) => <TableRow key={u.user_uuid}><TableCell className="font-mono">{u.username}</TableCell><TableCell>{u.email || "-"}</TableCell><TableCell>{u.enabled ? <Badge>yes</Badge> : "no"}</TableCell></TableRow>)}
                {users.length===0 && <TableRow><TableCell colSpan={3} className="text-center text-muted-foreground py-8">No users — seed creates superadmin</TableCell></TableRow>}
              </TableBody>
            </Table>
          </div>
          <div className="md:hidden grid gap-3">
            {users.map((u) => <Card key={u.user_uuid} className="p-3"><div className="font-mono font-bold">{u.username}</div><div className="text-xs">{u.email}</div></Card>)}
            {users.length===0 && <div className="text-center text-muted-foreground py-4">No users</div>}
          </div>
        </CardContent>
      </Card>
      <Card><CardHeader><CardTitle>RBAC Groups</CardTitle></CardHeader><CardContent className="text-sm text-muted-foreground">Groups: superadmin (all perms), user. Permissions: domain_view, extension_add, dialplan_view, gateway_view, etc. Stored in <code>groups</code> + <code>group_permissions</code>, checked via <code>require_permission</code> dep.</CardContent></Card>
      <Dialog open={open} onOpenChange={setOpen}>
        <DialogHeader><DialogTitle>Add User</DialogTitle></DialogHeader>
        <div className="space-y-3">
          <div><Label>Username *</Label><Input value={form.username} onChange={(e) => setForm({ ...form, username: e.target.value })} placeholder="alice" /></div>
          <div><Label>Email</Label><Input value={form.email} onChange={(e) => setForm({ ...form, email: e.target.value })} placeholder="alice@example.com" /></div>
          <div><Label>Password *</Label><Input type="password" value={form.password} onChange={(e) => setForm({ ...form, password: e.target.value })} /></div>
          <Button onClick={create} className="w-full">Create</Button>
        </div>
      </Dialog>
    </div>
  );
}
