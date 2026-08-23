import * as React from "react";
import { apiFetch } from "../lib/api";
import { DataTable } from "../components/DataTable";
import type { Column } from "../components/types";
import { FormDialog } from "../components/FormDialog";
import { PageHeader } from "../components/shell/PageHeader";
import { StatusBadge } from "../components/StatusBadge";
import { Button } from "../components/ui/button";
import { Input } from "../components/ui/input";
import { Label } from "../components/ui/label";
import { Card, CardContent, CardHeader, CardTitle } from "../components/ui/card";
import { useConfirm } from "../components/ui/confirm";
import { toast, toastError } from "../lib/toast";
import { useAsyncAction } from "../lib/useAsyncAction";
import { Pencil, Power, PowerOff, Trash2, Server } from "lucide-react";

type GwForm = {
  gateway: string; proxy: string; username: string; password: string; register: boolean; enabled: boolean;
  distinct_to: boolean; auth_username: string; realm: string; from_user: string; from_domain: string;
  register_proxy: string; outbound_proxy: string; expire_seconds: string; register_transport: string;
  contact_params: string; retry_seconds: string; extension: string; ping: string; ping_min: string; ping_max: string;
  contact_in_ping: boolean; caller_id_in_from: boolean; supress_cng: boolean; sip_cid_type: string; codec_prefs: string;
  channels: string; extension_in_contact: string; context: string; profile: string; hostname: string; description: string;
};

const emptyGw: GwForm = {
  gateway: "provider1", proxy: "sip.provider.com", username: "user", password: "pass", register: true, enabled: true,
  distinct_to: false, auth_username: "", realm: "", from_user: "", from_domain: "",
  register_proxy: "", outbound_proxy: "", expire_seconds: "800", register_transport: "udp",
  contact_params: "", retry_seconds: "30", extension: "", ping: "", ping_min: "", ping_max: "",
  contact_in_ping: false, caller_id_in_from: false, supress_cng: false, sip_cid_type: "", codec_prefs: "",
  channels: "", extension_in_contact: "", context: "public", profile: "external", hostname: "", description: "",
};

function toPayload(f: GwForm) {
  const p:any={};
  for (const [k,v] of Object.entries(f)) {
    if (v==="" || v===null) continue;
    if (typeof v==="boolean") { p[k]=v; continue; }
    if (["expire_seconds","retry_seconds","ping_min","ping_max","channels"].includes(k)) {
      const n=parseInt(v as string,10);
      if (!isNaN(n)) p[k]=n; else continue;
    } else p[k]=v;
  }
  return p;
}

type GwRow = any;

export function Gateways() {
  const confirm = useConfirm();
  const [reloadKey, setReloadKey] = React.useState(0);
  const reload = () => setReloadKey((k) => k + 1);
  const [open, setOpen] = React.useState(false);
  const [editing, setEditing] = React.useState<any | null>(null);
  const [form, setForm] = React.useState<GwForm>(emptyGw);
  const [showAdvanced, setShowAdvanced] = React.useState(false);
  const [verifyXml, setVerifyXml] = React.useState("");

  const columns: Column<GwRow>[] = [
    { key: "gateway", header: "Gateway", className: "font-mono font-medium" },
    { key: "proxy", header: "Proxy" },
    { key: "register", header: "Register", cell: (r) => (r.register ? "yes" : "no") },
    {
      key: "enabled",
      header: "Status",
      cell: (r) => <StatusBadge tone={r.enabled ? "ok" : "off"}>{r.enabled ? "enabled" : "disabled"}</StatusBadge>,
    },
  ];

  const startCreate = () => { setEditing(null); setForm(emptyGw); setOpen(true); setShowAdvanced(false); setVerifyXml(""); };
  const startEdit = (g: any) => {
    setEditing(g);
    setForm({
      gateway: g.gateway || "", proxy: g.proxy || "", username: g.username || "", password: "", register: !!g.register, enabled: !!g.enabled,
      distinct_to: !!g.distinct_to, auth_username: g.auth_username||"", realm: g.realm||"", from_user: g.from_user||"", from_domain: g.from_domain||"",
      register_proxy: g.register_proxy||"", outbound_proxy: g.outbound_proxy||"", expire_seconds: g.expire_seconds?String(g.expire_seconds):"800", register_transport: g.register_transport||"udp",
      contact_params: g.contact_params||"", retry_seconds: g.retry_seconds?String(g.retry_seconds):"30", extension: g.extension||"", ping: g.ping||"", ping_min: g.ping_min?String(g.ping_min):"", ping_max: g.ping_max?String(g.ping_max):"",
      contact_in_ping: !!g.contact_in_ping, caller_id_in_from: !!g.caller_id_in_from, supress_cng: !!g.supress_cng, sip_cid_type: g.sip_cid_type||"", codec_prefs: g.codec_prefs||"",
      channels: g.channels?String(g.channels):"", extension_in_contact: g.extension_in_contact||"", context: g.context||"public", profile: g.profile||"external", hostname: g.hostname||"", description: g.description||"",
    });
    setOpen(true); setShowAdvanced(false);
  };

  const saveFn = async () => {
    try {
      const payload = toPayload(form);
      if (editing && !form.password) delete payload.password;
      if (editing) {
        await apiFetch(`/gateways/${editing.gateway_uuid}`, { method: "PATCH", body: JSON.stringify(payload) });
        toast(`Gateway ${form.gateway} updated`, { variant: "success" });
      } else {
        await apiFetch("/gateways", { method: "POST", body: JSON.stringify(payload) });
        toast(`Gateway ${form.gateway} created`, { variant: "success" });
      }
      setOpen(false); setEditing(null); reload();
    } catch (err) {
      toastError(err);
    }
  };
  const { run: save, pending: saving } = useAsyncAction(saveFn);

  const toggleEnabled = async (g: any) => {
    try {
      await apiFetch(`/gateways/${g.gateway_uuid}`, { method: "PATCH", body: JSON.stringify({ enabled: !g.enabled }) });
      toast(`Gateway ${g.gateway} ${!g.enabled ? "enabled" : "disabled"}`);
      reload();
    } catch (err) { toastError(err); }
  };
  const deleteGw = async (g: any) => {
    const ok = await confirm({ title: `Delete gateway ${g.gateway}?`, description: "This cannot be undone.", confirmLabel: "Delete" });
    if (!ok) return;
    try {
      await apiFetch(`/gateways/${g.gateway_uuid}`, { method: "DELETE" });
      toast(`Gateway ${g.gateway} deleted`);
      reload();
    } catch (err) { toastError(err); }
  };
  const fetchXml = async () => {
    try {
      const me = await apiFetch("/auth/me").catch(()=>({domain_name:"test.local"}));
      const domain = (me as any).domain_name || "test.local";
      const res = await fetch(`/api/internal/freeswitch/xml?section=configuration&key_value=sofia.conf&domain=${domain}`, { headers: { "Authorization": "Basic "+btoa("freeswitch:ClueCon") }});
      setVerifyXml((await res.text()).slice(0,4000));
    } catch(e:any){ setVerifyXml("Failed "+e.message); }
  };

  return (
    <>
      <PageHeader
        title="Gateways / Trunks"
        description="SIP trunks and external carriers"
        actions={
          <>
            <Button variant="outline" onClick={fetchXml}>Verify Sofa XML</Button>
            <Button onClick={startCreate}>Add Gateway</Button>
          </>
        }
      />
      <DataTable<GwRow>
        resource="/gateways"
        columns={columns}
        rowKey={(r) => r.gateway_uuid}
        reloadKey={reloadKey}
        perPage={20}
        searchPlaceholder="Search gateways…"
        emptyTitle="No gateways"
        emptyDescription="Add your first SIP trunk"
        emptyIcon={Server}
        emptyAction={<Button onClick={startCreate}><Server className="h-4 w-4" /> Add Gateway</Button>}
        rowActions={(g) => (
          <div className="flex justify-end gap-1">
            <Button variant="ghost" size="sm" aria-label={`Edit ${g.gateway}`} title="Edit" onClick={() => startEdit(g)}><Pencil className="h-4 w-4" /> Edit</Button>
            <Button variant="ghost" size="sm" aria-label={g.enabled ? `Disable ${g.gateway}` : `Enable ${g.gateway}`} title={g.enabled ? "Disable" : "Enable"} onClick={() => toggleEnabled(g)}>{g.enabled ? <><PowerOff className="h-4 w-4" /> Disable</> : <><Power className="h-4 w-4" /> Enable</>}</Button>
            <Button variant="ghost" size="sm" className="text-destructive hover:text-destructive" aria-label={`Delete ${g.gateway}`} title="Delete" onClick={() => deleteGw(g)}><Trash2 className="h-4 w-4" /> Delete</Button>
          </div>
        )}
      />
      {verifyXml && <Card><CardHeader><CardTitle className="text-sm">Sofia Gateway XML — verifiable</CardTitle></CardHeader><CardContent><pre className="text-xs bg-muted p-2 rounded overflow-auto max-h-64">{verifyXml}</pre><Button variant="outline" className="mt-2" onClick={()=>setVerifyXml("")}>Close</Button></CardContent></Card>}
      <Card>
        <CardHeader><CardTitle>Profiles</CardTitle></CardHeader>
        <CardContent className="text-sm text-muted-foreground">Sofia SIP profiles — internal/external — per domain, via <code>GET /api/v1/sip_profiles</code> → FS xml <code>sofia.conf</code> fallback.</CardContent>
      </Card>

      <FormDialog open={open} onOpenChange={(v) => { setOpen(v); if (!v) setEditing(null); }} title={editing ? "Edit Gateway" : "Add Gateway"} onSubmit={save} pending={saving} submitLabel={editing ? "Save" : "Create"}>
        <div className="space-y-3 max-h-[65vh] overflow-auto pr-1">
          <div><Label>Gateway *</Label><Input value={form.gateway} onChange={(e) => setForm({ ...form, gateway: e.target.value })} placeholder="provider1" /></div>
          <div><Label>Proxy</Label><Input value={form.proxy} onChange={(e) => setForm({ ...form, proxy: e.target.value })} placeholder="sip.provider.com" /></div>
          <div className="grid grid-cols-2 gap-2"><div><Label>Username</Label><Input value={form.username} onChange={(e) => setForm({ ...form, username: e.target.value })} /></div><div><Label>Password</Label><Input type="password" value={form.password} onChange={(e) => setForm({ ...form, password: e.target.value })} placeholder={editing ? "leave blank to keep" : ""} /></div></div>
          <div className="flex items-center gap-2"><input type="checkbox" id="gw-register" checked={form.register} onChange={(e) => setForm({ ...form, register: e.target.checked })} className="h-4 w-4" /><Label htmlFor="gw-register" className="cursor-pointer">Register</Label><span className="ml-4 flex items-center gap-2"><input type="checkbox" id="gw-enabled" checked={form.enabled} onChange={(e) => setForm({ ...form, enabled: e.target.checked })} className="h-4 w-4" /><Label htmlFor="gw-enabled" className="cursor-pointer">Enabled</Label></span></div>
          <Button variant="outline" onClick={()=>setShowAdvanced(!showAdvanced)} className="w-full" type="button">{showAdvanced ? "Hide Advanced (30 fields)" : "Show Advanced (30 fields)"}</Button>
          {showAdvanced && (
            <div className="space-y-3 border rounded p-3 bg-muted/20">
              <div className="text-xs font-semibold">Auth & From</div>
              <div className="grid grid-cols-2 gap-2"><Input placeholder="auth_username" value={form.auth_username} onChange={e=>setForm({...form,auth_username:e.target.value})}/><Input placeholder="realm" value={form.realm} onChange={e=>setForm({...form,realm:e.target.value})}/></div>
              <div className="grid grid-cols-2 gap-2"><Input placeholder="from_user" value={form.from_user} onChange={e=>setForm({...form,from_user:e.target.value})}/><Input placeholder="from_domain" value={form.from_domain} onChange={e=>setForm({...form,from_domain:e.target.value})}/></div>
              <div className="grid grid-cols-2 gap-2"><Input placeholder="register_proxy" value={form.register_proxy} onChange={e=>setForm({...form,register_proxy:e.target.value})}/><Input placeholder="outbound_proxy" value={form.outbound_proxy} onChange={e=>setForm({...form,outbound_proxy:e.target.value})}/></div>
              <div className="text-xs font-semibold">Timers & Transport</div>
              <div className="grid grid-cols-3 gap-2"><Input placeholder="expire_seconds 800" value={form.expire_seconds} onChange={e=>setForm({...form,expire_seconds:e.target.value})}/><Input placeholder="retry_seconds 30" value={form.retry_seconds} onChange={e=>setForm({...form,retry_seconds:e.target.value})}/><Input placeholder="register_transport udp" value={form.register_transport} onChange={e=>setForm({...form,register_transport:e.target.value})}/></div>
              <Input placeholder="contact_params" value={form.contact_params} onChange={e=>setForm({...form,contact_params:e.target.value})}/>
              <div className="text-xs font-semibold">Ping & CNG</div>
              <div className="grid grid-cols-3 gap-2"><Input placeholder="ping" value={form.ping} onChange={e=>setForm({...form,ping:e.target.value})}/><Input placeholder="ping_min" value={form.ping_min} onChange={e=>setForm({...form,ping_min:e.target.value})}/><Input placeholder="ping_max" value={form.ping_max} onChange={e=>setForm({...form,ping_max:e.target.value})}/></div>
              <div className="flex gap-4"><label className="flex items-center gap-1 text-xs"><input type="checkbox" checked={form.contact_in_ping} onChange={e=>setForm({...form,contact_in_ping:e.target.checked})}/> contact_in_ping</label><label className="flex items-center gap-1 text-xs"><input type="checkbox" checked={form.caller_id_in_from} onChange={e=>setForm({...form,caller_id_in_from:e.target.checked})}/> caller_id_in_from</label><label className="flex items-center gap-1 text-xs"><input type="checkbox" checked={form.supress_cng} onChange={e=>setForm({...form,supress_cng:e.target.checked})}/> supress_cng</label></div>
              <div className="text-xs font-semibold">Codec & Channel</div>
              <Input placeholder="codec_prefs e.g. PCMA,PCMU" value={form.codec_prefs} onChange={e=>setForm({...form,codec_prefs:e.target.value})}/>
              <div className="grid grid-cols-2 gap-2"><Input placeholder="channels" value={form.channels} onChange={e=>setForm({...form,channels:e.target.value})}/><Input placeholder="sip_cid_type" value={form.sip_cid_type} onChange={e=>setForm({...form,sip_cid_type:e.target.value})}/></div>
              <div className="grid grid-cols-2 gap-2"><Input placeholder="extension" value={form.extension} onChange={e=>setForm({...form,extension:e.target.value})}/><Input placeholder="extension_in_contact" value={form.extension_in_contact} onChange={e=>setForm({...form,extension_in_contact:e.target.value})}/></div>
              <div className="grid grid-cols-2 gap-2"><Input placeholder="context public" value={form.context} onChange={e=>setForm({...form,context:e.target.value})}/><Input placeholder="profile external" value={form.profile} onChange={e=>setForm({...form,profile:e.target.value})}/></div>
              <Input placeholder="hostname" value={form.hostname} onChange={e=>setForm({...form,hostname:e.target.value})}/>
              <div className="flex gap-4"><label className="flex items-center gap-1 text-xs"><input type="checkbox" checked={form.distinct_to} onChange={e=>setForm({...form,distinct_to:e.target.checked})}/> distinct_to</label></div>
              <Input placeholder="description" value={form.description} onChange={e=>setForm({...form,description:e.target.value})}/>
            </div>
          )}
        </div>
      </FormDialog>
    </>
  );
}
