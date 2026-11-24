import * as React from "react";
import { apiFetch } from "../lib/api";
import { PageHeader } from "../components/shell/PageHeader";
import { Button } from "../components/ui/button";
import { Label } from "../components/ui/label";
import { Input } from "../components/ui/input";
import { Card, CardContent, CardHeader, CardTitle } from "../components/ui/card";
import { toast, toastError } from "../lib/toast";
import { useAsyncAction } from "../lib/useAsyncAction";

export function WebrtcSettings() {
  const [domainId, setDomainId] = React.useState("");
  const [form, setForm] = React.useState({ enabled: false, auth_mode: "extension", wss_url: "", sip_domain: "", ice_servers: "" });

  React.useEffect(() => {
    apiFetch("/auth/me")
      .then((me: any) => {
        const did = me?.domain_id ?? me?.domain_uuid ?? "";
        setDomainId(did);
        if (did) {
          apiFetch(`/domains/${did}/webrtc_settings`)
            .then((s: any) => setForm({
              enabled: !!s?.enabled,
              auth_mode: s?.auth_mode ?? "extension",
              wss_url: s?.wss_url ?? "",
              sip_domain: s?.sip_domain ?? "",
              ice_servers: typeof s?.ice_servers === "string" ? s.ice_servers : JSON.stringify(s?.ice_servers ?? []),
            }))
            .catch(() => {});
        }
      })
      .catch((err) => toastError(err));
  }, []);

  const { run: save, pending } = useAsyncAction(async () => {
    try {
      let ice = null;
      try { ice = form.ice_servers ? JSON.parse(form.ice_servers) : null; } catch { ice = null; }
      await apiFetch(`/domains/${domainId}/webrtc_settings`, {
        method: "PUT",
        body: JSON.stringify({ enabled: form.enabled, auth_mode: form.auth_mode, wss_url: form.wss_url || null, sip_domain: form.sip_domain || null, ice_servers: ice }),
      });
      toast("WebRTC settings saved", { variant: "success" });
    } catch (err) { toastError(err); }
  });

  return (
    <>
      <PageHeader title="WebRTC Settings" description="Browser phone transport per domain" />
      <Card>
        <CardHeader><CardTitle>Domain {domainId ? domainId.slice(0, 8) : ""}</CardTitle></CardHeader>
        <CardContent className="space-y-3 max-w-md">
          <div className="flex items-center gap-2">
            <input type="checkbox" id="wrtc-enabled" checked={form.enabled} onChange={(e) => setForm({ ...form, enabled: e.target.checked })} className="h-4 w-4" />
            <Label htmlFor="wrtc-enabled" className="cursor-pointer">Enabled</Label>
          </div>
          <div><Label>Auth mode (extension/webrtc/both)</Label><Input value={form.auth_mode} onChange={(e) => setForm({ ...form, auth_mode: e.target.value })} placeholder="extension" /></div>
          <div><Label>WSS URL</Label><Input value={form.wss_url} onChange={(e) => setForm({ ...form, wss_url: e.target.value })} placeholder="wss://example.com:7443" /></div>
          <div><Label>SIP domain</Label><Input value={form.sip_domain} onChange={(e) => setForm({ ...form, sip_domain: e.target.value })} placeholder="example.com" /></div>
          <div><Label>ICE servers (JSON)</Label><Input value={form.ice_servers} onChange={(e) => setForm({ ...form, ice_servers: e.target.value })} placeholder='[{"urls":"stun:stun.example.com"}]' /></div>
          <Button onClick={save} disabled={pending || !domainId}>{pending ? "Saving…" : "Save"}</Button>
        </CardContent>
      </Card>
    </>
  );
}
