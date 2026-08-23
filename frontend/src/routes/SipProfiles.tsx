import * as React from "react";
import { apiFetch } from "../lib/api";
import { Card, CardContent, CardHeader, CardTitle } from "../components/ui/card";
import { Button } from "../components/ui/button";
import { Input } from "../components/ui/input";
import { Label } from "../components/ui/label";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "../components/ui/table";
import { Dialog, DialogHeader, DialogTitle } from "../components/ui/dialog";
import { Badge } from "../components/ui/badge";
import { Pencil, Power, PowerOff, Trash2 } from "lucide-react";

export function SipProfiles() {
  const [data, setData] = React.useState<any[]>([]);
  const [open, setOpen] = React.useState(false);
  const [editing, setEditing] = React.useState<any | null>(null);
  const [form, setForm] = React.useState({ name: "internal", sip_port: 5060, sip_ip: "", enabled: true });
  const [statusMap, setStatusMap] = React.useState<Record<string, { fs_state: string; running: boolean }>>({});
  const [fsConnected, setFsConnected] = React.useState<boolean | null>(null);
  const load = () => apiFetch("/sip_profiles").then((r) => setData(r.data || [])).catch(() => {});
  const loadStatus = React.useCallback(() => {
    apiFetch("/sip_profiles/status")
      .then((r: any) => {
        setFsConnected(r.connected);
        const m: Record<string, { fs_state: string; running: boolean }> = {};
        (r.profiles || []).forEach((p: any) => {
          m[p.name] = { fs_state: p.fs_state, running: p.running };
          m[p.sip_profile_uuid] = { fs_state: p.fs_state, running: p.running };
        });
        setStatusMap(m);
      })
      .catch(() => setFsConnected(false));
  }, []);
  React.useEffect(() => { load(); loadStatus(); const id = setInterval(loadStatus, 30000); return () => clearInterval(id); }, [loadStatus]);

  const startCreate = () => { setEditing(null); setForm({ name: "internal", sip_port: 5060, sip_ip: "", enabled: true }); setOpen(true); };
  const startEdit = (p: any) => { setEditing(p); setForm({ name: p.name || "", sip_port: p.sip_port || 5060, sip_ip: p.sip_ip || "", enabled: !!p.enabled }); setOpen(true); };
  const createOrUpdate = async () => {
    try {
      if (editing) await apiFetch(`/sip_profiles/${editing.sip_profile_uuid}`, { method: "PATCH", body: JSON.stringify(form) });
      else await apiFetch("/sip_profiles", { method: "POST", body: JSON.stringify(form) });
      setOpen(false); setEditing(null); load(); loadStatus();
    } catch (e: any) { alert(e.message); }
  };
  const toggleEnabled = async (p: any) => {
    try { await apiFetch(`/sip_profiles/${p.sip_profile_uuid}`, { method: "PATCH", body: JSON.stringify({ enabled: !p.enabled }) }); load(); } catch (e: any) { alert(e.message); }
  };
  const deleteProfile = async (p: any) => {
    if (!confirm(`Delete SIP profile ${p.name}?`)) return;
    try { await apiFetch(`/sip_profiles/${p.sip_profile_uuid}`, { method: "DELETE" }); load(); } catch (e: any) { alert(e.message); }
  };

  return (
    <div className="p-4 space-y-4">
      <div className="flex justify-between items-center"><h1 className="text-2xl font-bold">SIP Profiles</h1><Button onClick={startCreate}>Add Profile</Button></div>
      <Card>
        <CardHeader className="flex flex-row items-center justify-between space-y-0">
          <CardTitle>Sofia profiles — internal/external</CardTitle>
          <span data-testid="sip-fs-connected" className={`text-xs px-2 py-1 rounded-full border ${fsConnected === true ? "bg-green-50 text-green-700 border-green-200" : fsConnected === false ? "bg-red-50 text-red-700 border-red-200" : "bg-muted text-muted-foreground"}`}>
            {fsConnected === true ? "" : fsConnected === false ? "" : "Checking…"}
          </span>
        </CardHeader>
        <CardContent>
          <div className="hidden md:block"><Table><TableHeader><TableRow><TableHead>Name</TableHead><TableHead>Port</TableHead><TableHead>Enabled</TableHead><TableHead>FS Status</TableHead><TableHead>Actions</TableHead></TableRow></TableHeader><TableBody>{data.map((p) => {
            const st = statusMap[p.name] || statusMap[p.sip_profile_uuid];
            const fsState = st?.fs_state || (fsConnected === false ? "UNKNOWN" : "—");
            const running = st?.running;
            let dot = "bg-gray-300";
            let textColor = "text-muted-foreground";
            if (fsConnected === false) { dot = "bg-gray-300"; textColor = "text-muted-foreground"; }
            else if (running === true) { dot = "bg-green-500"; textColor = "text-green-700"; }
            else if (running === false && st) { dot = "bg-red-500"; textColor = "text-red-700"; }
            return <TableRow key={p.sip_profile_uuid}><TableCell className="font-mono">{p.name}</TableCell><TableCell>{p.sip_port}</TableCell><TableCell>{p.enabled ? <Badge>enabled</Badge> : <Badge className="bg-muted text-muted-foreground">disabled</Badge>}</TableCell><TableCell><span data-testid={`sip-status-${p.name}`} className={`inline-flex items-center gap-1.5 text-xs ${textColor}`}><span className={`h-2 w-2 rounded-full ${dot} ${running ? "shadow-[0_0_4px_rgba(34,197,94,0.5)]" : ""}`} />{fsState}</span></TableCell><TableCell><div className="flex gap-1"><Button variant="ghost" aria-label={`Edit ${p.name}`} title="Edit" className="h-9 w-9 p-0" onClick={() => startEdit(p)}><Pencil className="h-4 w-4" /></Button><Button variant="ghost" aria-label={p.enabled ? `Disable ${p.name}` : `Enable ${p.name}`} title={p.enabled ? "Disable" : "Enable"} className="h-9 w-9 p-0" onClick={() => toggleEnabled(p)}>{p.enabled ? <PowerOff className="h-4 w-4" /> : <Power className="h-4 w-4" />}</Button><Button variant="ghost" aria-label={`Delete ${p.name}`} title="Delete" className="h-9 w-9 p-0 text-destructive hover:text-destructive hover:bg-destructive/10" onClick={() => deleteProfile(p)}><Trash2 className="h-4 w-4" /></Button></div></TableCell></TableRow>;
          })}{data.length===0 && <TableRow><TableCell colSpan={5} className="text-center py-8 text-muted-foreground">No profiles</TableCell></TableRow>}</TableBody></Table></div>
          <div className="md:hidden grid gap-2">{data.map((p) => {
            const st = statusMap[p.name] || statusMap[p.sip_profile_uuid];
            const fsState = st?.fs_state || (fsConnected === false ? "UNKNOWN" : "—");
            const running = st?.running;
            let dot = "bg-gray-300";
            if (running === true) dot = "bg-green-500";
            else if (running === false && st) dot = "bg-red-500";
            return <Card key={p.sip_profile_uuid} className="p-3"><div className="flex justify-between items-center"><div className="font-mono font-bold">{p.name}:{p.sip_port}</div><span className="inline-flex items-center gap-1 text-xs"><span className={`h-2 w-2 rounded-full ${dot}`} />{fsState}</span></div><div className="mt-1 flex gap-2"><Badge>{p.enabled ? "enabled" : "disabled"}</Badge></div><div className="flex gap-2 mt-2"><Button variant="ghost" aria-label={`Edit ${p.name}`} title="Edit" className="flex-1 h-9" onClick={() => startEdit(p)}><Pencil className="h-4 w-4" /></Button><Button variant="ghost" aria-label={p.enabled ? `Disable ${p.name}` : `Enable ${p.name}`} title={p.enabled ? "Disable" : "Enable"} className="flex-1 h-9" onClick={() => toggleEnabled(p)}>{p.enabled ? <PowerOff className="h-4 w-4" /> : <Power className="h-4 w-4" />}</Button><Button variant="ghost" aria-label={`Delete ${p.name}`} title="Delete" className="flex-1 h-9 text-destructive hover:text-destructive hover:bg-destructive/10" onClick={() => deleteProfile(p)}><Trash2 className="h-4 w-4" /></Button></div></Card>;
          })}{data.length===0 && <div className="text-center py-4 text-muted-foreground">No profiles</div>}</div>
        </CardContent>
      </Card>
      <Dialog open={open} onOpenChange={(v) => { setOpen(v); if (!v) setEditing(null); }}>
        <DialogHeader><DialogTitle>{editing ? "Edit SIP Profile" : "Add SIP Profile"}</DialogTitle></DialogHeader>
        <div className="space-y-3"><div><Label>Name *</Label><Input value={form.name} onChange={(e) => setForm({ ...form, name: e.target.value })} /></div><div><Label>Port</Label><Input type="number" value={form.sip_port} onChange={(e) => setForm({ ...form, sip_port: Number(e.target.value) })} /></div><div><Label>SIP IP</Label><Input value={form.sip_ip} onChange={(e) => setForm({ ...form, sip_ip: e.target.value })} placeholder="auto" /></div><div className="flex items-center gap-2"><input type="checkbox" id="sip-enabled" checked={form.enabled} onChange={(e) => setForm({ ...form, enabled: e.target.checked })} className="h-4 w-4" /><Label htmlFor="sip-enabled" className="cursor-pointer">{form.enabled ? "Enabled" : "Disabled"}</Label></div><Button onClick={createOrUpdate} className="w-full">{editing ? "Save" : "Create"}</Button></div>
      </Dialog>
    </div>
  );
}
