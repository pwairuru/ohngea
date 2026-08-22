import * as React from "react";
import { apiFetch } from "../lib/api";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "../components/ui/table";
import { Card, CardContent, CardHeader, CardTitle } from "../components/ui/card";
import { Button } from "../components/ui/button";
import { Input } from "../components/ui/input";
import { Label } from "../components/ui/label";
import { Dialog, DialogHeader, DialogTitle } from "../components/ui/dialog";
import { Badge } from "../components/ui/badge";
import {Pencil, Power, PowerOff, Trash2, Eye, ChevronsLeft, ChevronLeft, ChevronRight, ChevronsRight} from "lucide-react";

type FullForm = {
  extension: string; number_alias: string; password: string; accountcode: string;
  effective_caller_id_name: string; effective_caller_id_number: string;
  outbound_caller_id_name: string; outbound_caller_id_number: string;
  emergency_caller_id_name: string; emergency_caller_id_number: string;
  directory_first_name: string; directory_last_name: string; directory_visible: boolean; directory_exten_visible: boolean;
  max_registrations: string; limit_max: string; limit_destination: string;
  user_context: string; toll_allow: string; call_timeout: string; call_group: string;
  call_screen_enabled: boolean; user_record: string; hold_music: string;
  auth_acl: string; cidr: string; sip_force_contact: string; sip_force_expires: string;
  nibble_account: string; mwi_account: string; sip_bypass_media: string; absolute_codec_string: string;
  force_ping: boolean; dial_string: string;
  do_not_disturb: boolean; forward_all_destination: string; forward_all_enabled: boolean;
  forward_busy_destination: string; forward_busy_enabled: boolean;
  forward_no_answer_destination: string; forward_no_answer_enabled: boolean;
  forward_user_not_registered_destination: string; forward_user_not_registered_enabled: boolean;
  extension_language: string; extension_dialect: string; extension_voice: string; extension_type: string;
  description: string; enabled: boolean;
};

const emptyForm: FullForm = {
  extension: "", number_alias: "", password: "", accountcode: "",
  effective_caller_id_name: "", effective_caller_id_number: "",
  outbound_caller_id_name: "", outbound_caller_id_number: "",
  emergency_caller_id_name: "", emergency_caller_id_number: "",
  directory_first_name: "", directory_last_name: "", directory_visible: false, directory_exten_visible: false,
  max_registrations: "", limit_max: "5", limit_destination: "!USER_BUSY",
  user_context: "", toll_allow: "", call_timeout: "30", call_group: "",
  call_screen_enabled: false, user_record: "", hold_music: "",
  auth_acl: "", cidr: "", sip_force_contact: "", sip_force_expires: "",
  nibble_account: "", mwi_account: "", sip_bypass_media: "", absolute_codec_string: "",
  force_ping: false, dial_string: "",
  do_not_disturb: false, forward_all_destination: "", forward_all_enabled: false,
  forward_busy_destination: "", forward_busy_enabled: false,
  forward_no_answer_destination: "", forward_no_answer_enabled: false,
  forward_user_not_registered_destination: "", forward_user_not_registered_enabled: false,
  extension_language: "", extension_dialect: "", extension_voice: "", extension_type: "default",
  description: "", enabled: true,
};

function toApiPayload(f: FullForm) {
  const p:any = {};
  for (const [k,v] of Object.entries(f)) {
    if (v === "" || v === null) continue;
    // booleans keep false only if explicitly toggled? we send all booleans
    if (typeof v === "boolean") { p[k]=v; continue; }
    p[k]=v;
  }
  // call_timeout and sip_force_expires as int if present
  if (p.call_timeout) p.call_timeout = parseInt(p.call_timeout,10);
  if (p.sip_force_expires) p.sip_force_expires = parseInt(p.sip_force_expires,10);
  return p;
}

export function Extensions() {
  const [data, setData] = React.useState<any[]>([]);
  const [total, setTotal] = React.useState(0);
  const [q, setQ] = React.useState("");
  const [page, setPage] = React.useState(1);
  const [open, setOpen] = React.useState(false);
  const [step, setStep] = React.useState(1);
  const [form, setForm] = React.useState<FullForm>(emptyForm);
  const [editing, setEditing] = React.useState<any | null>(null);
  const [csvOpen, setCsvOpen] = React.useState(false);
  const [csvText, setCsvText] = React.useState("extension,name\n1001,Alice\n1002,Bob");
  const [verifyXml, setVerifyXml] = React.useState<string>("");
  const [showAdvanced, setShowAdvanced] = React.useState(false);

  const load = React.useCallback(() => {
    apiFetch(`/extensions?page=${page}&per_page=10&q=${encodeURIComponent(q)}`)
      .then((res) => { setData(res.data || []); setTotal(res.total || 0); })
      .catch(() => setData([]));
  }, [page, q]);

  React.useEffect(() => { load(); }, [load]);

  const create = async () => {
    try {
      if (editing) {
        const payload = toApiPayload(form);
        // don't send empty password on edit (keep existing)
        if (!form.password) delete payload.password;
        await apiFetch(`/extensions/${editing.extension_uuid}`, {
          method: "PATCH",
          body: JSON.stringify(payload),
        });
        setOpen(false); setStep(1); setEditing(null);
        setForm(emptyForm); setShowAdvanced(false);
        load();
        return;
      }
      const me = await apiFetch("/auth/me");
      const domain_id = me.domain_id;
      if (!domain_id) { alert("No domain found — create domain first"); return; }
      const payload = { ...toApiPayload(form), domain_id };
      await apiFetch("/extensions", { method: "POST", body: JSON.stringify(payload) });
      setOpen(false); setStep(1); setForm(emptyForm); setShowAdvanced(false);
      load();
    } catch (e: any) { alert(e.message); }
  };

  const startEdit = (e: any) => {
    setEditing(e);
    setForm({
      extension: e.extension || "",
      number_alias: e.number_alias || "",
      password: "", // blank = keep
      effective_caller_id_name: e.effective_caller_id_name || "",
      effective_caller_id_number: e.effective_caller_id_number || "",
      outbound_caller_id_name: e.outbound_caller_id_name || "",
      outbound_caller_id_number: e.outbound_caller_id_number || "",
      emergency_caller_id_name: e.emergency_caller_id_name || "",
      emergency_caller_id_number: e.emergency_caller_id_number || "",
      accountcode: e.accountcode || "",
      directory_first_name: e.directory_first_name || "",
      directory_last_name: e.directory_last_name || "",
      directory_visible: !!e.directory_visible,
      directory_exten_visible: !!e.directory_exten_visible,
      max_registrations: e.max_registrations || "",
      limit_max: e.limit_max || "5",
      limit_destination: e.limit_destination || "!USER_BUSY",
      user_context: e.user_context || "",
      toll_allow: e.toll_allow || "",
      call_timeout: String(e.call_timeout || "30"),
      call_group: e.call_group || "",
      call_screen_enabled: !!e.call_screen_enabled,
      user_record: e.user_record || "",
      hold_music: e.hold_music || "",
      auth_acl: e.auth_acl || "",
      cidr: e.cidr || "",
      sip_force_contact: e.sip_force_contact || "",
      sip_force_expires: e.sip_force_expires ? String(e.sip_force_expires) : "",
      nibble_account: e.nibble_account || "",
      mwi_account: e.mwi_account || "",
      sip_bypass_media: e.sip_bypass_media || "",
      absolute_codec_string: e.absolute_codec_string || "",
      force_ping: !!e.force_ping,
      dial_string: e.dial_string || "",
      do_not_disturb: !!e.do_not_disturb,
      forward_all_destination: e.forward_all_destination || "",
      forward_all_enabled: !!e.forward_all_enabled,
      forward_busy_destination: e.forward_busy_destination || "",
      forward_busy_enabled: !!e.forward_busy_enabled,
      forward_no_answer_destination: e.forward_no_answer_destination || "",
      forward_no_answer_enabled: !!e.forward_no_answer_enabled,
      forward_user_not_registered_destination: e.forward_user_not_registered_destination || "",
      forward_user_not_registered_enabled: !!e.forward_user_not_registered_enabled,
      extension_language: e.extension_language || "",
      extension_dialect: e.extension_dialect || "",
      extension_voice: e.extension_voice || "",
      extension_type: e.extension_type || "default",
      description: e.description || "",
      enabled: !!e.enabled,
    });
    setStep(1);
    setOpen(true);
    setShowAdvanced(false);
  };

  const startCreate = () => {
    setEditing(null);
    setForm(emptyForm);
    setStep(1);
    setOpen(true);
    setShowAdvanced(false);
    setVerifyXml("");
  };

  const toggleEnabled = async (e: any) => {
    try {
      await apiFetch(`/extensions/${e.extension_uuid}`, { method: "PATCH", body: JSON.stringify({ enabled: !e.enabled }) });
      load();
    } catch (err: any) { alert(err.message); }
  };

  const deleteExt = async (e: any) => {
    if (!confirm(`Delete extension ${e.extension}?`)) return;
    try {
      await apiFetch(`/extensions/${e.extension_uuid}`, { method: "DELETE" });
      load();
    } catch (err: any) { alert(err.message); }
  };

  const fetchXml = async (ext: string) => {
    try {
      const me = await apiFetch("/auth/me");
      // domain from me? need domain name; fallback to user_context or guess
      // For verify we call internal xml with basic auth via apiFetch with header? Instead call via fetch with credentials
      // Use apiFetch's underlying fetch with Basic? Simpler call public endpoint without auth for demo (shows cached xml)
      // We will call /api/internal/freeswitch/xml via apiFetch which may not need auth in dev
      const domain = me.domain_name || me.domain || "test.local";
      const res = await fetch(`/api/internal/freeswitch/xml?section=directory&key_value=${ext}@${domain}`, {
        headers: { "Authorization": "Basic " + btoa("freeswitch:ClueCon") }
      });
      const txt = await res.text();
      setVerifyXml(txt.slice(0, 3000));
    } catch (e:any) { setVerifyXml("Failed: "+e.message); }
  };

  const parseCsv = async () => {
    const lines = csvText.split("\n").filter(Boolean);
    const rows = lines.slice(1).map((l) => {
      const v = l.split(",");
      return { extension: v[0], effective_caller_id_name: v[1] };
    });
    try {
      const me = await apiFetch("/auth/me");
      const domain_id = me.domain_id;
      const payload = rows.map((r) => ({ ...r, domain_id, enabled: true }));
      await apiFetch("/extensions:batch", { method: "POST", body: JSON.stringify(payload) });
      setCsvOpen(false); load();
    } catch (e: any) { alert(e.message); }
  };

  return (
    <div className="p-4 space-y-4">
      <div className="flex flex-wrap items-center justify-between gap-2">
        <h1 className="text-2xl font-bold">Extensions</h1>
        <div className="flex gap-2">
          <Button variant="outline" onClick={() => setCsvOpen(true)}>CSV Import</Button>
          <Button onClick={startCreate}>Add Extension</Button>
        </div>
      </div>

      <div className="flex gap-2">
        <Input value={q} onChange={(e) => setQ(e.target.value)} placeholder="Search extensions, name, number..." className="flex-1" />
        <Button onClick={() => { setPage(1); load(); }}>Search</Button>
      </div>

      <div className="hidden md:block">
        <Card>
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Extension</TableHead>
                <TableHead>Name</TableHead>
                <TableHead>Number</TableHead>
                <TableHead>Status</TableHead>
                <TableHead>Actions</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {data.map((e) => (
                <TableRow key={e.extension_uuid}>
                  <TableCell className="font-mono font-bold">{e.extension}</TableCell>
                  <TableCell>{e.effective_caller_id_name}</TableCell>
                  <TableCell>{e.effective_caller_id_number || e.number_alias || "-"}</TableCell>
                  <TableCell>{e.enabled ? <Badge>enabled</Badge> : <Badge className="bg-muted text-muted-foreground">disabled</Badge>}</TableCell>
                  <TableCell>
                    <div className="flex gap-1">
                      <Button variant="ghost" aria-label={`Verify ${e.extension}`} title="Verify FS XML" className="h-9 w-9 p-0" onClick={() => fetchXml(e.extension)}><Eye className="h-4 w-4" /></Button>
                      <Button variant="ghost" aria-label={`Edit ${e.extension}`} title="Edit" className="h-9 w-9 p-0" onClick={() => startEdit(e)}><Pencil className="h-4 w-4" /></Button>
                      <Button variant="ghost" aria-label={e.enabled ? `Disable ${e.extension}` : `Enable ${e.extension}`} title={e.enabled ? "Disable" : "Enable"} className="h-9 w-9 p-0" onClick={() => toggleEnabled(e)}>{e.enabled ? <PowerOff className="h-4 w-4" /> : <Power className="h-4 w-4" />}</Button>
                      <Button variant="ghost" aria-label={`Delete ${e.extension}`} title="Delete" className="h-9 w-9 p-0 text-destructive hover:text-destructive hover:bg-destructive/10" onClick={() => deleteExt(e)}><Trash2 className="h-4 w-4" /></Button>
                    </div>
                  </TableCell>
                </TableRow>
              ))}
              {data.length === 0 && <TableRow><TableCell colSpan={5} className="text-center text-muted-foreground py-8">No extensions — try Add or CSV Import</TableCell></TableRow>}
            </TableBody>
          </Table>
        </Card>
      </div>

      <div className="grid gap-3 md:hidden" data-testid="extensions-cards">
        {data.map((e) => (
          <Card key={e.extension_uuid} data-card className="p-4">
            <div className="flex justify-between items-start">
              <div className="font-mono font-bold text-lg">{e.extension}</div>
              <Badge>{e.enabled ? "enabled" : "disabled"}</Badge>
            </div>
            <div className="text-sm font-medium">{e.effective_caller_id_name}</div>
            <div className="text-xs text-muted-foreground">{e.effective_caller_id_number || "no number"}</div>
            <div className="flex gap-2 mt-3">
              <Button variant="ghost" aria-label={`Edit ${e.extension}`} title="Edit" className="flex-1 h-9" onClick={() => startEdit(e)}><Pencil className="h-4 w-4" /></Button>
              <Button variant="ghost" aria-label={e.enabled ? `Disable ${e.extension}` : `Enable ${e.extension}`} title={e.enabled ? "Disable" : "Enable"} className="flex-1 h-9" onClick={() => toggleEnabled(e)}>{e.enabled ? <PowerOff className="h-4 w-4" /> : <Power className="h-4 w-4" />}</Button>
              <Button variant="ghost" aria-label={`Delete ${e.extension}`} title="Delete" className="flex-1 h-9 text-destructive hover:text-destructive hover:bg-destructive/10" onClick={() => deleteExt(e)}><Trash2 className="h-4 w-4" /></Button>
            </div>
          </Card>
        ))}
        {data.length === 0 && <Card className="p-8 text-center text-muted-foreground">No extensions</Card>}
      </div>

      {verifyXml && (
        <Card><CardHeader><CardTitle className="text-sm">FreeSWITCH XML (directory) — verifiable</CardTitle></CardHeader>
        <CardContent><pre className="text-xs bg-muted p-2 rounded overflow-auto max-h-64">{verifyXml}</pre><Button variant="outline" className="mt-2" onClick={()=>setVerifyXml("")}>Close</Button></CardContent></Card>
      )}

      <div className="flex items-center justify-between text-sm">
        <span>Total: {total}</span>
        <div className="flex items-center gap-1">
              <Button variant="ghost" size="icon" aria-label="First page" title="First page" className="h-9 w-9 p-0" disabled={page <= 1} onClick={() => setPage(1)}><ChevronsLeft className="h-4 w-4" /></Button>
              <Button variant="ghost" size="icon" aria-label="Previous page" title="Previous page" className="h-9 w-9 p-0" disabled={page <= 1} onClick={() => setPage((p) => Math.max(1, p - 1))}><ChevronLeft className="h-4 w-4" /></Button>
              <span className="px-2 text-sm">Page {page} of {Math.max(1, Math.ceil(total / 10))}</span>
              <Button variant="ghost" size="icon" aria-label="Next page" title="Next page" className="h-9 w-9 p-0" disabled={page * 10 >= total} onClick={() => setPage((p) => p + 1)}><ChevronRight className="h-4 w-4" /></Button>
              <Button variant="ghost" size="icon" aria-label="Last page" title="Last page" className="h-9 w-9 p-0" disabled={page * 10 >= total} onClick={() => setPage(Math.max(1, Math.ceil(total / 10)))}><ChevronsRight className="h-4 w-4" /></Button>
            </div>
      </div>

      <Dialog open={open} onOpenChange={(v) => { setOpen(v); if (!v) { setEditing(null); setStep(1); setShowAdvanced(false); } }}>
        <DialogHeader><DialogTitle>{editing ? "Edit Extension" : "Add Extension"} — Step {step} of 3</DialogTitle></DialogHeader>
        <div className="flex gap-1 mb-4">
          {[1, 2, 3].map((s) => <div key={s} className={`h-1 flex-1 rounded ${s <= step ? "bg-primary" : "bg-muted"}`} />)}
        </div>
        {step === 1 && (
          <div className="space-y-3 max-h-[60vh] overflow-auto pr-1">
            <div><Label>Extension *</Label><Input value={form.extension} onChange={(e) => setForm({ ...form, extension: e.target.value })} placeholder="1001" pattern="\d{2,7}" /></div>
            <div><Label>Display Name *</Label><Input value={form.effective_caller_id_name} onChange={(e) => setForm({ ...form, effective_caller_id_name: e.target.value })} placeholder="Alice Smith" /></div>
            <div><Label>Password {editing ? "(leave blank to keep)" : "(auto-generated if blank)"}</Label><Input value={form.password} onChange={(e) => setForm({ ...form, password: e.target.value })} placeholder={editing ? "leave blank to keep" : "20-char auto"} type="password" /></div>
            <div><Label>Number Alias</Label><Input value={form.number_alias} onChange={(e) => setForm({ ...form, number_alias: e.target.value })} placeholder="Optional alias" /></div>
            <Button onClick={() => setStep(2)} className="w-full">Next: Caller ID</Button>
          </div>
        )}
        {step === 2 && (
          <div className="space-y-3 max-h-[60vh] overflow-auto pr-1">
            <div><Label>Caller ID Number</Label><Input value={form.effective_caller_id_number} onChange={(e) => setForm({ ...form, effective_caller_id_number: e.target.value })} placeholder="+15551234567" /></div>
            <div className="grid grid-cols-2 gap-2">
              <div><Label>Outbound CID Name</Label><Input value={form.outbound_caller_id_name} onChange={(e)=>setForm({...form,outbound_caller_id_name:e.target.value})}/></div>
              <div><Label>Outbound CID Number</Label><Input value={form.outbound_caller_id_number} onChange={(e)=>setForm({...form,outbound_caller_id_number:e.target.value})}/></div>
            </div>
            <div className="grid grid-cols-2 gap-2">
              <div><Label>Emergency CID Name</Label><Input value={form.emergency_caller_id_name} onChange={(e)=>setForm({...form,emergency_caller_id_name:e.target.value})}/></div>
              <div><Label>Emergency CID Number</Label><Input value={form.emergency_caller_id_number} onChange={(e)=>setForm({...form,emergency_caller_id_number:e.target.value})}/></div>
            </div>
            <div className="flex gap-2"><Button variant="outline" onClick={() => setShowAdvanced(!showAdvanced)} className="flex-1">{showAdvanced ? "Hide Advanced" : "Show Advanced (50 fields)"}</Button></div>
            {showAdvanced && (
              <div className="space-y-3 border rounded p-3 bg-muted/20">
                <div className="text-xs font-semibold">Directory</div>
                <div className="grid grid-cols-2 gap-2"><Input placeholder="First name" value={form.directory_first_name} onChange={e=>setForm({...form,directory_first_name:e.target.value})}/><Input placeholder="Last name" value={form.directory_last_name} onChange={e=>setForm({...form,directory_last_name:e.target.value})}/></div>
                <div className="flex gap-4"><label className="flex items-center gap-1 text-xs"><input type="checkbox" checked={form.directory_visible} onChange={e=>setForm({...form,directory_visible:e.target.checked})}/> Visible</label><label className="flex items-center gap-1 text-xs"><input type="checkbox" checked={form.directory_exten_visible} onChange={e=>setForm({...form,directory_exten_visible:e.target.checked})}/> Ext visible</label></div>
                <div className="text-xs font-semibold">Limits & Context</div>
                <div className="grid grid-cols-2 gap-2"><div><Label className="text-xs">Max Registrations</Label><Input value={form.max_registrations} onChange={e=>setForm({...form,max_registrations:e.target.value})} /></div><div><Label className="text-xs">Call Timeout</Label><Input value={form.call_timeout} onChange={e=>setForm({...form,call_timeout:e.target.value})} /></div></div>
                <div className="grid grid-cols-2 gap-2"><Input placeholder="limit_max 5" value={form.limit_max} onChange={e=>setForm({...form,limit_max:e.target.value})}/><Input placeholder="limit_destination" value={form.limit_destination} onChange={e=>setForm({...form,limit_destination:e.target.value})}/></div>
                <div><Label className="text-xs">User Context</Label><Input value={form.user_context} onChange={e=>setForm({...form,user_context:e.target.value})} placeholder="domain name" /></div>
                <div className="grid grid-cols-2 gap-2"><Input placeholder="toll_allow" value={form.toll_allow} onChange={e=>setForm({...form,toll_allow:e.target.value})}/><Input placeholder="call_group" value={form.call_group} onChange={e=>setForm({...form,call_group:e.target.value})}/></div>
                <div className="grid grid-cols-2 gap-2"><Input placeholder="hold_music" value={form.hold_music} onChange={e=>setForm({...form,hold_music:e.target.value})}/><Input placeholder="user_record" value={form.user_record} onChange={e=>setForm({...form,user_record:e.target.value})}/></div>
                <div className="flex gap-2"><label className="flex items-center gap-1 text-xs"><input type="checkbox" checked={form.call_screen_enabled} onChange={e=>setForm({...form,call_screen_enabled:e.target.checked})}/> Call Screen</label><label className="flex items-center gap-1 text-xs"><input type="checkbox" checked={form.force_ping} onChange={e=>setForm({...form,force_ping:e.target.checked})}/> Force Ping</label><label className="flex items-center gap-1 text-xs"><input type="checkbox" checked={form.do_not_disturb} onChange={e=>setForm({...form,do_not_disturb:e.target.checked})}/> DND</label></div>
                <div className="text-xs font-semibold">SIP Advanced (hidden by default)</div>
                <div className="grid grid-cols-2 gap-2"><Input placeholder="auth_acl" value={form.auth_acl} onChange={e=>setForm({...form,auth_acl:e.target.value})}/><Input placeholder="cidr" value={form.cidr} onChange={e=>setForm({...form,cidr:e.target.value})}/></div>
                <div className="grid grid-cols-2 gap-2"><Input placeholder="sip_force_contact" value={form.sip_force_contact} onChange={e=>setForm({...form,sip_force_contact:e.target.value})}/><Input placeholder="sip_force_expires" value={form.sip_force_expires} onChange={e=>setForm({...form,sip_force_expires:e.target.value})}/></div>
                <div className="grid grid-cols-2 gap-2"><Input placeholder="mwi_account" value={form.mwi_account} onChange={e=>setForm({...form,mwi_account:e.target.value})}/><Input placeholder="nibble_account" value={form.nibble_account} onChange={e=>setForm({...form,nibble_account:e.target.value})}/></div>
                <div className="grid grid-cols-2 gap-2"><Input placeholder="sip_bypass_media" value={form.sip_bypass_media} onChange={e=>setForm({...form,sip_bypass_media:e.target.value})}/><Input placeholder="absolute_codec_string" value={form.absolute_codec_string} onChange={e=>setForm({...form,absolute_codec_string:e.target.value})}/></div>
                <div><Label className="text-xs">Dial String</Label><Input value={form.dial_string} onChange={e=>setForm({...form,dial_string:e.target.value})} placeholder="{sip_invite_domain...}" /></div>
                <div className="text-xs font-semibold">Language / Type</div>
                <div className="grid grid-cols-3 gap-2"><Input placeholder="language" value={form.extension_language} onChange={e=>setForm({...form,extension_language:e.target.value})}/><Input placeholder="dialect" value={form.extension_dialect} onChange={e=>setForm({...form,extension_dialect:e.target.value})}/><Input placeholder="voice" value={form.extension_voice} onChange={e=>setForm({...form,extension_voice:e.target.value})}/></div>
                <Input placeholder="extension_type" value={form.extension_type} onChange={e=>setForm({...form,extension_type:e.target.value})}/>
                <div className="text-xs font-semibold">Forwarding</div>
                <div className="grid grid-cols-2 gap-2">
                  <label className="flex items-center gap-1 text-xs"><input type="checkbox" checked={form.forward_all_enabled} onChange={e=>setForm({...form,forward_all_enabled:e.target.checked})}/> All</label><Input placeholder="All dest" value={form.forward_all_destination} onChange={e=>setForm({...form,forward_all_destination:e.target.value})}/>
                  <label className="flex items-center gap-1 text-xs"><input type="checkbox" checked={form.forward_busy_enabled} onChange={e=>setForm({...form,forward_busy_enabled:e.target.checked})}/> Busy</label><Input placeholder="Busy dest" value={form.forward_busy_destination} onChange={e=>setForm({...form,forward_busy_destination:e.target.value})}/>
                  <label className="flex items-center gap-1 text-xs"><input type="checkbox" checked={form.forward_no_answer_enabled} onChange={e=>setForm({...form,forward_no_answer_enabled:e.target.checked})}/> No Answer</label><Input placeholder="NoAns dest" value={form.forward_no_answer_destination} onChange={e=>setForm({...form,forward_no_answer_destination:e.target.value})}/>
                  <label className="flex items-center gap-1 text-xs"><input type="checkbox" checked={form.forward_user_not_registered_enabled} onChange={e=>setForm({...form,forward_user_not_registered_enabled:e.target.checked})}/> Not Registered</label><Input placeholder="NotReg dest" value={form.forward_user_not_registered_destination} onChange={e=>setForm({...form,forward_user_not_registered_destination:e.target.value})}/>
                </div>
                <div><Label className="text-xs">Description</Label><Input value={form.description} onChange={e=>setForm({...form,description:e.target.value})} /></div>
                <div><Label className="text-xs">Accountcode</Label><Input value={form.accountcode} onChange={e=>setForm({...form,accountcode:e.target.value})} /></div>
              </div>
            )}
            <div className="flex gap-2"><Button variant="outline" onClick={() => setStep(1)} className="flex-1">Back</Button><Button onClick={() => setStep(3)} className="flex-1">Next: Review</Button></div>
          </div>
        )}
        {step === 3 && (
          <div className="space-y-3 max-h-[60vh] overflow-auto pr-1">
            <Card><CardContent className="pt-4 text-sm space-y-1"><div>Extension: <span className="font-mono">{form.extension}</span> {form.number_alias && `(${form.number_alias})`}</div><div>Name: {form.effective_caller_id_name}</div><div>Number: {form.effective_caller_id_number || "-"}</div><div>Password: {form.password ? "••••" : "(auto)"}</div><div>Context: {form.user_context || "(domain)"}</div><div>Call Group: {form.call_group || "-"}</div><div className="flex items-center gap-2 pt-2"><input type="checkbox" id="ext-enabled" checked={form.enabled} onChange={(e) => setForm({ ...form, enabled: e.target.checked })} className="h-4 w-4" /><Label htmlFor="ext-enabled" className="cursor-pointer">{form.enabled ? "Enabled" : "Disabled"}</Label></div></CardContent></Card>
            {showAdvanced && <Button variant="outline" onClick={()=>setShowAdvanced(false)}>Hide Advanced</Button>}
            {!showAdvanced && <Button variant="ghost" onClick={()=>setShowAdvanced(true)} className="w-full">Show Advanced summary ({Object.values(toApiPayload(form)).length} fields)</Button>}
            {showAdvanced && (
              <Card><CardContent className="pt-4 text-xs space-y-1 max-h-40 overflow-auto"><pre>{JSON.stringify(toApiPayload(form), null, 2)}</pre></CardContent></Card>
            )}
            <div className="flex gap-2"><Button variant="outline" onClick={() => setStep(2)} className="flex-1">Back</Button><Button onClick={create} className="flex-1">{editing ? "Save" : "Create"}</Button></div>
          </div>
        )}
      </Dialog>

      <Dialog open={csvOpen} onOpenChange={setCsvOpen}>
        <DialogHeader><DialogTitle>CSV Import Wizard</DialogTitle></DialogHeader>
        <div className="space-y-3">
          <div className="text-sm text-muted-foreground">Paste CSV with headers extension,name — preview below. Inspired by 3CX/Cisco search-first tables.</div>
          <textarea value={csvText} onChange={(e) => setCsvText(e.target.value)} rows={6} className="w-full border rounded p-2 font-mono text-sm" />
          <div className="rounded border p-2 bg-muted/30 max-h-32 overflow-auto text-xs">
            <div className="font-medium mb-1">Preview (3 rows)</div>
            {csvText.split("\n").slice(0, 4).map((l, i) => <div key={i} className={i === 0 ? "font-bold" : ""}>{l}</div>)}
          </div>
          <div className="flex gap-2"><Button variant="outline" onClick={() => setCsvOpen(false)} className="flex-1">Cancel</Button><Button onClick={parseCsv} className="flex-1">Import</Button></div>
        </div>
      </Dialog>
    </div>
  );
}
