import * as React from "react";
import { apiFetch } from "../lib/api";
import { Card, CardContent, CardHeader, CardTitle } from "../components/ui/card";
import { Button } from "../components/ui/button";
import { Input } from "../components/ui/input";
import { Label } from "../components/ui/label";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "../components/ui/table";
import { Dialog, DialogHeader, DialogTitle } from "../components/ui/dialog";
import { Badge } from "../components/ui/badge";

export function SipProfiles() {
  const [data, setData] = React.useState<any[]>([]);
  const [open, setOpen] = React.useState(false);
  const [form, setForm] = React.useState({ name: "internal", sip_port: 5060 });
  const load = () => apiFetch("/sip_profiles").then((r) => setData(r.data || [])).catch(() => {});
  React.useEffect(() => { load(); }, []);
  const create = async () => { await apiFetch("/sip_profiles", { method: "POST", body: JSON.stringify(form) }); setOpen(false); load(); };
  return (
    <div className="p-4 space-y-4">
      <div className="flex justify-between items-center"><h1 className="text-2xl font-bold">SIP Profiles</h1><Button onClick={() => setOpen(true)}>Add Profile</Button></div>
      <Card>
        <CardHeader><CardTitle>Sofia profiles — internal/external</CardTitle></CardHeader>
        <CardContent>
          <div className="hidden md:block"><Table><TableHeader><TableRow><TableHead>Name</TableHead><TableHead>Port</TableHead><TableHead>Enabled</TableHead></TableRow></TableHeader><TableBody>{data.map((p) => <TableRow key={p.sip_profile_uuid}><TableCell className="font-mono">{p.name}</TableCell><TableCell>{p.sip_port}</TableCell><TableCell>{p.enabled ? <Badge>yes</Badge> : "no"}</TableCell></TableRow>)}{data.length===0 && <TableRow><TableCell colSpan={3} className="text-center py-8 text-muted-foreground">No profiles</TableCell></TableRow>}</TableBody></Table></div>
          <div className="md:hidden grid gap-2">{data.map((p) => <Card key={p.sip_profile_uuid} className="p-3"><div className="font-mono font-bold">{p.name}:{p.sip_port}</div><Badge>{p.enabled ? "enabled" : "disabled"}</Badge></Card>)}{data.length===0 && <div className="text-center py-4 text-muted-foreground">No profiles</div>}</div>
        </CardContent>
      </Card>
      <Dialog open={open} onOpenChange={setOpen}>
        <DialogHeader><DialogTitle>Add SIP Profile</DialogTitle></DialogHeader>
        <div className="space-y-3"><div><Label>Name</Label><Input value={form.name} onChange={(e) => setForm({ ...form, name: e.target.value })} /></div><div><Label>Port</Label><Input type="number" value={form.sip_port} onChange={(e) => setForm({ ...form, sip_port: Number(e.target.value) })} /></div><Button onClick={create} className="w-full">Create</Button></div>
      </Dialog>
    </div>
  );
}
