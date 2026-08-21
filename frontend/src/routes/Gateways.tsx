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
  const [form, setForm] = React.useState({ gateway: "provider1", proxy: "sip.provider.com", username: "user", password: "pass", register: true });
  const load = () => apiFetch("/gateways").then((r) => setData(r.data || [])).catch(() => {});
  React.useEffect(() => { load(); }, []);
  const create = async () => {
    try { await apiFetch("/gateways", { method: "POST", body: JSON.stringify(form) }); setOpen(false); load(); } catch (e: any) { alert(e.message); }
  };
  return (
    <div className="p-4 space-y-4">
      <div className="flex justify-between items-center"><h1 className="text-2xl font-bold">Gateways / Trunks</h1><Button onClick={() => setOpen(true)}>Add Gateway</Button></div>
      <Card>
        <CardHeader><CardTitle>SIP Trunks — Sofia profiles</CardTitle></CardHeader>
        <CardContent>
          <div className="hidden md:block">
            <Table>
              <TableHeader><TableRow><TableHead>Gateway</TableHead><TableHead>Proxy</TableHead><TableHead>Register</TableHead><TableHead>Enabled</TableHead></TableRow></TableHeader>
              <TableBody>
                {data.map((g) => <TableRow key={g.gateway_uuid}><TableCell className="font-mono font-bold">{g.gateway}</TableCell><TableCell>{g.proxy || "-"}</TableCell><TableCell>{g.register ? "yes" : "no"}</TableCell><TableCell>{g.enabled ? <Badge>yes</Badge> : "no"}</TableCell></TableRow>)}
                {data.length===0 && <TableRow><TableCell colSpan={4} className="text-center text-muted-foreground py-8">No gateways — add SIP trunk via wizard</TableCell></TableRow>}
              </TableBody>
            </Table>
          </div>
          <div className="md:hidden grid gap-3">
            {data.map((g) => <Card key={g.gateway_uuid} className="p-3"><div className="font-mono font-bold">{g.gateway}</div><div className="text-xs">{g.proxy}</div><Badge className="mt-1">{g.enabled ? "enabled" : "disabled"}</Badge></Card>)}
            {data.length===0 && <div className="text-center text-muted-foreground py-4">No gateways</div>}
          </div>
        </CardContent>
      </Card>
      <Card>
        <CardHeader><CardTitle>Profiles</CardTitle></CardHeader>
        <CardContent className="text-sm text-muted-foreground">Sofia SIP profiles — internal/external — per domain, via <code>GET /api/v1/sip_profiles</code> → FS xml <code>sofia.conf</code> fallback.</CardContent>
      </Card>
      <Dialog open={open} onOpenChange={setOpen}>
        <DialogHeader><DialogTitle>Add Gateway</DialogTitle></DialogHeader>
        <div className="space-y-3">
          <div><Label>Gateway *</Label><Input value={form.gateway} onChange={(e) => setForm({ ...form, gateway: e.target.value })} placeholder="provider1" /></div>
          <div><Label>Proxy</Label><Input value={form.proxy} onChange={(e) => setForm({ ...form, proxy: e.target.value })} placeholder="sip.provider.com" /></div>
          <div className="grid grid-cols-2 gap-2"><div><Label>Username</Label><Input value={form.username} onChange={(e) => setForm({ ...form, username: e.target.value })} /></div><div><Label>Password</Label><Input type="password" value={form.password} onChange={(e) => setForm({ ...form, password: e.target.value })} /></div></div>
          <Button onClick={create} className="w-full">Create</Button>
        </div>
      </Dialog>
    </div>
  );
}
