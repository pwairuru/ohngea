import * as React from "react";
import { apiFetch } from "../lib/api";
import { Card, CardContent, CardHeader, CardTitle } from "../components/ui/card";
import { Button } from "../components/ui/button";
import { Input } from "../components/ui/input";
import { Label } from "../components/ui/label";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "../components/ui/table";
import { Dialog, DialogHeader, DialogTitle } from "../components/ui/dialog";
import { Badge } from "../components/ui/badge";
import { Pencil, Phone, Power, PowerOff, Trash2 } from "lucide-react";

export function Domains() {
  const [data, setData] = React.useState<any[]>([]);
  const [open, setOpen] = React.useState(false);
  const [editing, setEditing] = React.useState<any | null>(null);
  const [name, setName] = React.useState("tenant-" + Math.floor(Math.random() * 1000) + ".example.com");
  const [enabled, setEnabled] = React.useState(true);
  const [webrtcOpen, setWebrtcOpen] = React.useState(false);
  const [webrtcDomain, setWebrtcDomain] = React.useState<any | null>(null);
  const [webrtcForm, setWebrtcForm] = React.useState({ enabled: false, auth_mode: "extension", wss_url: "", sip_domain: "", ice_servers: "" });
  const [webrtcSaving, setWebrtcSaving] = React.useState(false);

  const load = () => apiFetch("/domains").then((d) => setData(Array.isArray(d) ? d : d.data || [])).catch(() => {});
  React.useEffect(() => { load(); }, []);

  const startCreate = () => { setEditing(null); setName("tenant-" + Math.floor(Math.random() * 1000) + ".example.com"); setEnabled(true); setOpen(true); };
  const startEdit = (d: any) => { setEditing(d); setName(d.domain_name || ""); setEnabled(!!d.enabled); setOpen(true); };
  const createOrUpdate = async () => {
    try {
      if (editing) {
        await apiFetch(`/domains/${editing.domain_id}`, { method: "PATCH", body: JSON.stringify({ domain_name: name, enabled }) });
      } else {
        await apiFetch("/domains", { method: "POST", body: JSON.stringify({ domain_name: name, enabled }) });
      }
      setOpen(false); setEditing(null); load();
    } catch (e: any) { alert(e.message); }
  };
  const toggleEnabled = async (d: any) => {
    try { await apiFetch(`/domains/${d.domain_id}`, { method: "PATCH", body: JSON.stringify({ enabled: !d.enabled }) }); load(); } catch (e: any) { alert(e.message); }
  };
  const deleteDomain = async (d: any) => {
    if (!confirm(`Delete domain ${d.domain_name}? This will remove all tenant data.`)) return;
    try { await apiFetch(`/domains/${d.domain_id}`, { method: "DELETE" }); load(); } catch (e: any) { alert(e.message); }
  };
  const openWebrtc = async (d: any) => {
    setWebrtcDomain(d);
    try {
      const s = await apiFetch(`/domains/${d.domain_id}/webrtc_settings`);
      setWebrtcForm({
        enabled: !!s.enabled,
        auth_mode: s.auth_mode || "extension",
        wss_url: s.wss_url || "",
        sip_domain: s.sip_domain || "",
        ice_servers: s.ice_servers ? JSON.stringify(s.ice_servers) : "",
      });
    } catch {
      setWebrtcForm({ enabled: false, auth_mode: "extension", wss_url: "ws://localhost:5066", sip_domain: d.domain_name, ice_servers: "" });
    }
    setWebrtcOpen(true);
  };
  const saveWebrtc = async () => {
    if (!webrtcDomain) return;
    setWebrtcSaving(true);
    try {
      let ice: any = null;
      if (webrtcForm.ice_servers.trim()) {
        try { ice = JSON.parse(webrtcForm.ice_servers); } catch { throw new Error("ice_servers must be valid JSON array"); }
      }
      await apiFetch(`/domains/${webrtcDomain.domain_id}/webrtc_settings`, {
        method: "PUT",
        body: JSON.stringify({
          enabled: webrtcForm.enabled,
          auth_mode: webrtcForm.auth_mode,
          wss_url: webrtcForm.wss_url || null,
          sip_domain: webrtcForm.sip_domain || null,
          ice_servers: ice,
        }),
      });
      setWebrtcOpen(false);
    } catch (e: any) { alert(e.message); } finally { setWebrtcSaving(false); }
  };

  return (
    <div className="p-4 space-y-4">
      <div className="flex justify-between items-center"><h1 className="text-2xl font-bold">Tenants (Domains)</h1><Button onClick={startCreate}>Add Tenant</Button></div>
      <Card>
        <CardHeader><CardTitle>Thousands tenants — hash partitioned</CardTitle></CardHeader>
        <CardContent>
          <div className="hidden md:block">
            <Table>
              <TableHeader><TableRow><TableHead>Domain</TableHead><TableHead>Enabled</TableHead><TableHead>Created</TableHead><TableHead>Actions</TableHead></TableRow></TableHeader>
              <TableBody>
                {data.map((d) => <TableRow key={d.domain_id}><TableCell className="font-mono">{d.domain_name}</TableCell><TableCell>{d.enabled ? <Badge>active</Badge> : <Badge className="bg-muted text-muted-foreground">disabled</Badge>}</TableCell><TableCell className="text-xs">{d.created_at ? new Date(d.created_at).toLocaleDateString() : "-"}</TableCell><TableCell><div className="flex gap-1"><Button variant="ghost" aria-label={`Edit ${d.domain_name}`} title="Edit" className="h-9 w-9 p-0" onClick={() => startEdit(d)}><Pencil className="h-4 w-4" /></Button><Button variant="ghost" aria-label={`WebPhone ${d.domain_name}`} title="WebPhone" className="h-9 w-9 p-0" onClick={() => openWebrtc(d)}><Phone className="h-4 w-4" /></Button><Button variant="ghost" aria-label={d.enabled ? `Disable ${d.domain_name}` : `Enable ${d.domain_name}`} title={d.enabled ? "Disable" : "Enable"} className="h-9 w-9 p-0" onClick={() => toggleEnabled(d)}>{d.enabled ? <PowerOff className="h-4 w-4" /> : <Power className="h-4 w-4" />}</Button><Button variant="ghost" aria-label={`Delete ${d.domain_name}`} title="Delete" className="h-9 w-9 p-0 text-destructive hover:text-destructive hover:bg-destructive/10" onClick={() => deleteDomain(d)}><Trash2 className="h-4 w-4" /></Button></div></TableCell></TableRow>)}
                {data.length===0 && <TableRow><TableCell colSpan={4} className="text-center text-muted-foreground py-8">No tenants — create first via subscription</TableCell></TableRow>}
              </TableBody>
            </Table>
          </div>
          <div className="md:hidden grid gap-3">
            {data.map((d) => <Card key={d.domain_id} className="p-3"><div className="flex justify-between"><div className="font-mono font-bold">{d.domain_name}</div><Badge className={d.enabled ? "" : "bg-muted text-muted-foreground"}>{d.enabled ? "active" : "disabled"}</Badge></div><div className="text-xs">{d.created_at}</div><div className="flex gap-2 mt-2"><Button variant="ghost" aria-label={`Edit ${d.domain_name}`} title="Edit" className="flex-1 h-9" onClick={() => startEdit(d)}><Pencil className="h-4 w-4" /></Button><Button variant="ghost" aria-label={`WebPhone ${d.domain_name}`} title="WebPhone" className="flex-1 h-9" onClick={() => openWebrtc(d)}><Phone className="h-4 w-4" /></Button><Button variant="ghost" aria-label={d.enabled ? `Disable ${d.domain_name}` : `Enable ${d.domain_name}`} title={d.enabled ? "Disable" : "Enable"} className="flex-1 h-9" onClick={() => toggleEnabled(d)}>{d.enabled ? <PowerOff className="h-4 w-4" /> : <Power className="h-4 w-4" />}</Button><Button variant="ghost" aria-label={`Delete ${d.domain_name}`} title="Delete" className="flex-1 h-9 text-destructive hover:text-destructive hover:bg-destructive/10" onClick={() => deleteDomain(d)}><Trash2 className="h-4 w-4" /></Button></div></Card>)}
            {data.length===0 && <div className="text-center text-muted-foreground py-4">No tenants</div>}
          </div>
        </CardContent>
      </Card>
      <Dialog open={open} onOpenChange={(v) => { setOpen(v); if (!v) setEditing(null); }}>
        <DialogHeader><DialogTitle>{editing ? "Edit Tenant" : "Add Tenant (Subscription)"}</DialogTitle></DialogHeader>
        <div className="space-y-3">
          <div><Label>Domain *</Label><Input value={name} onChange={(e) => setName(e.target.value)} placeholder="tenant.example.com" /></div>
          <div className="flex items-center gap-2"><input type="checkbox" id="domain-enabled" checked={enabled} onChange={(e) => setEnabled(e.target.checked)} className="h-4 w-4" /><Label htmlFor="domain-enabled" className="cursor-pointer">{enabled ? "Enabled" : "Disabled"}</Label></div>
          <div className="text-xs text-muted-foreground">Will seed dialplans + superadmin, attach hash partition.</div>
          <Button onClick={createOrUpdate} className="w-full">{editing ? "Save" : "Create"}</Button>
        </div>
      </Dialog>
      <Dialog open={webrtcOpen} onOpenChange={setWebrtcOpen}>
        <DialogHeader><DialogTitle>WebPhone — {webrtcDomain?.domain_name}</DialogTitle></DialogHeader>
        <div className="space-y-3">
          <div className="flex items-center gap-2"><input type="checkbox" id="webrtc-enabled" checked={webrtcForm.enabled} onChange={(e) => setWebrtcForm({ ...webrtcForm, enabled: e.target.checked })} className="h-4 w-4" /><Label htmlFor="webrtc-enabled" className="cursor-pointer">Enabled</Label></div>
          <div><Label>Auth Mode</Label><select value={webrtcForm.auth_mode} onChange={(e) => setWebrtcForm({ ...webrtcForm, auth_mode: e.target.value })} className="w-full border rounded px-3 py-2"><option value="extension">extension</option><option value="webrtc">webrtc</option><option value="both">both</option></select><p className="text-xs text-muted-foreground mt-1">extension = existing SIP password, webrtc = custom SIP creds, both = either. Tenants set via PUT.</p></div>
          <div><Label>WSS URL</Label><Input value={webrtcForm.wss_url} onChange={(e) => setWebrtcForm({ ...webrtcForm, wss_url: e.target.value })} placeholder="wss://pbx.example.com/wss or ws://localhost:5066" /><p className="text-xs text-muted-foreground mt-1">Browser SIP over Secure WebSocket. Use wss:// for prod, ws://localhost for dev.</p></div>
          <div><Label>SIP Domain (optional)</Label><Input value={webrtcForm.sip_domain} onChange={(e) => setWebrtcForm({ ...webrtcForm, sip_domain: e.target.value })} placeholder={webrtcDomain?.domain_name || "test.example.com"} /></div>
          <div><Label>ICE Servers (JSON)</Label><Input value={webrtcForm.ice_servers} onChange={(e) => setWebrtcForm({ ...webrtcForm, ice_servers: e.target.value })} placeholder='[{"urls":"stun:stun.l.google.com:19302"}]' /></div>
          <Button onClick={saveWebrtc} disabled={webrtcSaving} className="w-full">{webrtcSaving ? "Saving..." : "Save WebPhone Settings"}</Button>
          <div className="text-xs text-muted-foreground">Topnav phone button shows registration dot: green=registered, yellow=connecting, red=failed, grey=unconfigured. Popup: configure → dialpad → call. Inbound + outbound via SIP over WSS.</div>
        </div>
      </Dialog>
    </div>
  );
}
