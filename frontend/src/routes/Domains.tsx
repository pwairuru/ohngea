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
  const [name, setName] = React.useState("tenant-" + Math.floor(Math.random() * 1000) + ".example.com");

  const load = () => apiFetch("/domains").then((d) => setData(Array.isArray(d) ? d : d.data || [])).catch(() => {});
  React.useEffect(() => { load(); }, []);

  const create = async () => {
    try { await apiFetch("/domains", { method: "POST", body: JSON.stringify({ domain_name: name, enabled: true }) }); setOpen(false); load(); } catch (e: any) { alert(e.message); }
  };

  return (
    <div className="p-4 space-y-4">
      <div className="flex justify-between items-center"><h1 className="text-2xl font-bold">Tenants (Domains)</h1><Button onClick={() => setOpen(true)}>Add Tenant</Button></div>
      <Card>
        <CardHeader><CardTitle>Thousands tenants — hash partitioned</CardTitle></CardHeader>
        <CardContent>
          <div className="hidden md:block">
            <Table>
              <TableHeader><TableRow><TableHead>Domain</TableHead><TableHead>Enabled</TableHead><TableHead>Created</TableHead></TableRow></TableHeader>
              <TableBody>
                {data.map((d) => <TableRow key={d.domain_id}><TableCell className="font-mono">{d.domain_name}</TableCell><TableCell>{d.enabled ? <Badge>active</Badge> : "disabled"}</TableCell><TableCell className="text-xs">{new Date(d.created_at).toLocaleDateString()}</TableCell></TableRow>)}
                {data.length===0 && <TableRow><TableCell colSpan={3} className="text-center text-muted-foreground py-8">No tenants — create first via subscription</TableCell></TableRow>}
              </TableBody>
            </Table>
          </div>
          <div className="md:hidden grid gap-3">
            {data.map((d) => <Card key={d.domain_id} className="p-3"><div className="font-mono font-bold">{d.domain_name}</div><div className="text-xs">{d.created_at}</div></Card>)}
            {data.length===0 && <div className="text-center text-muted-foreground py-4">No tenants</div>}
          </div>
        </CardContent>
      </Card>
      <Dialog open={open} onOpenChange={setOpen}>
        <DialogHeader><DialogTitle>Add Tenant (Subscription)</DialogTitle></DialogHeader>
        <div className="space-y-3">
          <div><Label>Domain *</Label><Input value={name} onChange={(e) => setName(e.target.value)} placeholder="tenant.example.com" /></div>
          <div className="text-xs text-muted-foreground">Will seed dialplans + superadmin, attach hash partition.</div>
          <Button onClick={create} className="w-full">Create</Button>
        </div>
      </Dialog>
    </div>
  );
}
