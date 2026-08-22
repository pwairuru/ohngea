import * as React from "react";
import { apiFetch } from "../lib/api";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "../components/ui/table";
import { Card, CardContent, CardHeader, CardTitle } from "../components/ui/card";
import { Button } from "../components/ui/button";
import { Input } from "../components/ui/input";
import { Label } from "../components/ui/label";
import { Dialog, DialogHeader, DialogTitle } from "../components/ui/dialog";
import { Badge } from "../components/ui/badge";

export function Gateways() {
  const [data, setData] = React.useState<any[]>([]);
  const [open, setOpen] = React.useState(false);
  const [editing, setEditing] = React.useState<any | null>(null);
  const [form, setForm] = React.useState({ gateway: "provider1", proxy: "sip.provider.com", username: "user", password: "pass", register: true, enabled: true });
  const load = () => apiFetch("/gateways").then((r) => setData(r.data || [])).catch(() => {});
  React.useEffect(() => { load(); }, []);

  const startCreate = () => {
    setEditing(null);
    setForm({ gateway: "provider1", proxy: "sip.provider.com", username: "user", password: "pass", register: true, enabled: true });
    setOpen(true);
  };
  const startEdit = (g: any) => {
    setEditing(g);
    setForm({ gateway: g.gateway || "", proxy: g.proxy || "", username: g.username || "", password: "", register: !!g.register, enabled: !!g.enabled });
    setOpen(true);
  };
  const createOrUpdate = async () => {
    try {
      if (editing) {
        await apiFetch(`/gateways/${editing.gateway_uuid}`, { method: "PATCH", body: JSON.stringify(form) });
      } else {
        await apiFetch("/gateways", { method: "POST", body: JSON.stringify(form) });
      }
      setOpen(false); setEditing(null); load();
    } catch (e: any) { alert(e.message); }
  };
  const toggleEnabled = async (g: any) => {
    try { await apiFetch(`/gateways/${g.gateway_uuid}`, { method: "PATCH", body: JSON.stringify({ enabled: !g.enabled }) }); load(); } catch (e: any) { alert(e.message); }
  };
  const deleteGw = async (g: any) => {
    if (!confirm(`Delete gateway ${g.gateway}?`)) return;
    try { await apiFetch(`/gateways/${g.gateway_uuid}`, { method: "DELETE" }); load(); } catch (e: any) { alert(e.message); }
  };

  return (
    <div className="p-4 space-y-4">
      <div className="flex justify-between items-center"><h1 className="text-2xl font-bold">Gateways / Trunks</h1><Button onClick={startCreate}>Add Gateway</Button></div>
      <Card>
        <CardHeader><CardTitle>SIP Trunks — Sofia profiles</CardTitle></CardHeader>
        <CardContent>
          <div className="hidden md:block">
            <Table>
              <TableHeader><TableRow><TableHead>Gateway</TableHead><TableHead>Proxy</TableHead><TableHead>Register</TableHead><TableHead>Enabled</TableHead><TableHead>Actions</TableHead></TableRow></TableHeader>
              <TableBody>
                {data.map((g) => <TableRow key={g.gateway_uuid}><TableCell className="font-mono font-bold">{g.gateway}</TableCell><TableCell>{g.proxy || "-"}</TableCell><TableCell>{g.register ? "yes" : "no"}</TableCell><TableCell>{g.enabled ? <Badge>enabled</Badge> : <Badge className="bg-muted text-muted-foreground">disabled</Badge>}</TableCell><TableCell><div className="flex gap-1"><Button variant="outline" className="h-8" onClick={() => startEdit(g)}>Edit</Button><Button variant="outline" className="h-8" onClick={() => toggleEnabled(g)}>{g.enabled ? "Disable" : "Enable"}</Button><Button variant="outline" className="h-8 text-red-600 border-red-200 hover:bg-red-50" onClick={() => deleteGw(g)}>Delete</Button></div></TableCell></TableRow>)}
                {data.length===0 && <TableRow><TableCell colSpan={5} className="text-center text-muted-foreground py-8">No gateways — add SIP trunk via wizard</TableCell></TableRow>}
              </TableBody>
            </Table>
          </div>
          <div className="md:hidden grid gap-3">
            {data.map((g) => <Card key={g.gateway_uuid} className="p-3"><div className="flex justify-between"><div className="font-mono font-bold">{g.gateway}</div><Badge className={g.enabled ? "" : "bg-muted text-muted-foreground"}>{g.enabled ? "enabled" : "disabled"}</Badge></div><div className="text-xs">{g.proxy}</div><div className="text-xs">register: {g.register ? "yes" : "no"}</div><div className="flex gap-2 mt-2"><Button variant="outline" className="flex-1 h-8" onClick={() => startEdit(g)}>Edit</Button><Button variant="outline" className="flex-1 h-8" onClick={() => toggleEnabled(g)}>{g.enabled ? "Disable" : "Enable"}</Button><Button variant="outline" className="flex-1 h-8 text-red-600" onClick={() => deleteGw(g)}>Delete</Button></div></Card>)}
            {data.length===0 && <div className="text-center text-muted-foreground py-4">No gateways</div>}
          </div>
        </CardContent>
      </Card>
      <Card>
        <CardHeader><CardTitle>Profiles</CardTitle></CardHeader>
        <CardContent className="text-sm text-muted-foreground">Sofia SIP profiles — internal/external — per domain, via <code>GET /api/v1/sip_profiles</code> → FS xml <code>sofia.conf</code> fallback.</CardContent>
      </Card>
      <Dialog open={open} onOpenChange={(v) => { setOpen(v); if (!v) setEditing(null); }}>
        <DialogHeader><DialogTitle>{editing ? "Edit Gateway" : "Add Gateway"}</DialogTitle></DialogHeader>
        <div className="space-y-3">
          <div><Label>Gateway *</Label><Input value={form.gateway} onChange={(e) => setForm({ ...form, gateway: e.target.value })} placeholder="provider1" /></div>
          <div><Label>Proxy</Label><Input value={form.proxy} onChange={(e) => setForm({ ...form, proxy: e.target.value })} placeholder="sip.provider.com" /></div>
          <div className="grid grid-cols-2 gap-2"><div><Label>Username</Label><Input value={form.username} onChange={(e) => setForm({ ...form, username: e.target.value })} /></div><div><Label>Password</Label><Input type="password" value={form.password} onChange={(e) => setForm({ ...form, password: e.target.value })} placeholder={editing ? "leave blank to keep" : ""} /></div></div>
          <div className="flex items-center gap-2"><input type="checkbox" id="gw-register" checked={form.register} onChange={(e) => setForm({ ...form, register: e.target.checked })} className="h-4 w-4" /><Label htmlFor="gw-register" className="cursor-pointer">Register</Label><span className="ml-4 flex items-center gap-2"><input type="checkbox" id="gw-enabled" checked={form.enabled} onChange={(e) => setForm({ ...form, enabled: e.target.checked })} className="h-4 w-4" /><Label htmlFor="gw-enabled" className="cursor-pointer">Enabled</Label></span></div>
          <Button onClick={createOrUpdate} className="w-full">{editing ? "Save" : "Create"}</Button>
        </div>
      </Dialog>
    </div>
  );
}
