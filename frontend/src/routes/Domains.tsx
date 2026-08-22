import * as React from "react";
import { apiFetch } from "../lib/api";
import { Card, CardContent, CardHeader, CardTitle } from "../components/ui/card";
import { Button } from "../components/ui/button";
import { Input } from "../components/ui/input";
import { Label } from "../components/ui/label";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "../components/ui/table";
import { Dialog, DialogHeader, DialogTitle } from "../components/ui/dialog";
import { Badge } from "../components/ui/badge";

export function Domains() {
  const [data, setData] = React.useState<any[]>([]);
  const [open, setOpen] = React.useState(false);
  const [editing, setEditing] = React.useState<any | null>(null);
  const [name, setName] = React.useState("tenant-" + Math.floor(Math.random() * 1000) + ".example.com");
  const [enabled, setEnabled] = React.useState(true);

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
                {data.map((d) => <TableRow key={d.domain_id}><TableCell className="font-mono">{d.domain_name}</TableCell><TableCell>{d.enabled ? <Badge>active</Badge> : <Badge className="bg-muted text-muted-foreground">disabled</Badge>}</TableCell><TableCell className="text-xs">{d.created_at ? new Date(d.created_at).toLocaleDateString() : "-"}</TableCell><TableCell><div className="flex gap-1"><Button variant="outline" className="h-8" onClick={() => startEdit(d)}>Edit</Button><Button variant="outline" className="h-8" onClick={() => toggleEnabled(d)}>{d.enabled ? "Disable" : "Enable"}</Button><Button variant="outline" className="h-8 text-red-600 border-red-200" onClick={() => deleteDomain(d)}>Delete</Button></div></TableCell></TableRow>)}
                {data.length===0 && <TableRow><TableCell colSpan={4} className="text-center text-muted-foreground py-8">No tenants — create first via subscription</TableCell></TableRow>}
              </TableBody>
            </Table>
          </div>
          <div className="md:hidden grid gap-3">
            {data.map((d) => <Card key={d.domain_id} className="p-3"><div className="flex justify-between"><div className="font-mono font-bold">{d.domain_name}</div><Badge className={d.enabled ? "" : "bg-muted text-muted-foreground"}>{d.enabled ? "active" : "disabled"}</Badge></div><div className="text-xs">{d.created_at}</div><div className="flex gap-2 mt-2"><Button variant="outline" className="flex-1 h-8" onClick={() => startEdit(d)}>Edit</Button><Button variant="outline" className="flex-1 h-8" onClick={() => toggleEnabled(d)}>{d.enabled ? "Disable" : "Enable"}</Button><Button variant="outline" className="flex-1 h-8 text-red-600" onClick={() => deleteDomain(d)}>Delete</Button></div></Card>)}
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
    </div>
  );
}
