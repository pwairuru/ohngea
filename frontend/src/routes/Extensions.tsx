import * as React from "react";
import { apiFetch } from "../lib/api";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "../components/ui/table";
import { Card, CardContent, CardHeader, CardTitle } from "../components/ui/card";
import { Button } from "../components/ui/button";
import { Input } from "../components/ui/input";
import { Label } from "../components/ui/label";
import { Dialog, DialogHeader, DialogTitle } from "../components/ui/dialog";
import { Badge } from "../components/ui/badge";

export function Extensions() {
  const [data, setData] = React.useState<any[]>([]);
  const [total, setTotal] = React.useState(0);
  const [q, setQ] = React.useState("");
  const [page, setPage] = React.useState(1);
  const [open, setOpen] = React.useState(false);
  const [step, setStep] = React.useState(1);
  const [form, setForm] = React.useState({ extension: "", effective_caller_id_name: "", effective_caller_id_number: "", enabled: true });
  const [csvOpen, setCsvOpen] = React.useState(false);
  const [csvText, setCsvText] = React.useState("extension,name\n1001,Alice\n1002,Bob");

  const load = React.useCallback(() => {
    apiFetch(`/extensions?page=${page}&per_page=10&q=${encodeURIComponent(q)}`)
      .then((res) => { setData(res.data || []); setTotal(res.total || 0); })
      .catch(() => setData([]));
  }, [page, q]);

  React.useEffect(() => { load(); }, [load]);

  const create = async () => {
    // for demo: need domain_id — fetch first domain
    try {
      const domains = await apiFetch("/domains");
      const domain_id = domains[0]?.domain_id || domains.data?.[0]?.domain_id;
      if (!domain_id) { alert("No domain found — create domain first"); return; }
      await apiFetch("/extensions", { method: "POST", body: JSON.stringify({ ...form, domain_id }) });
      setOpen(false); setStep(1); setForm({ extension: "", effective_caller_id_name: "", effective_caller_id_number: "", enabled: true });
      load();
    } catch (e: any) { alert(e.message); }
  };

  const parseCsv = async () => {
    const lines = csvText.split("\n").filter(Boolean);
    const header = lines[0].split(",");
    const rows = lines.slice(1).map((l) => {
      const v = l.split(",");
      return { extension: v[0], effective_caller_id_name: v[1] };
    });
    try {
      const domains = await apiFetch("/domains");
      const domain_id = domains[0]?.domain_id || domains.data?.[0]?.domain_id;
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
          <Button onClick={() => setOpen(true)}>Add Extension</Button>
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
                  <TableCell>{e.effective_caller_id_number || "-"}</TableCell>
                  <TableCell>{e.enabled ? <Badge>enabled</Badge> : <Badge className="bg-muted text-muted-foreground">disabled</Badge>}</TableCell>
                  <TableCell><Button variant="outline" className="h-8">Edit</Button></TableCell>
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
          </Card>
        ))}
        {data.length === 0 && <Card className="p-8 text-center text-muted-foreground">No extensions</Card>}
      </div>

      <div className="flex items-center justify-between text-sm">
        <span>Total: {total}</span>
        <div className="flex gap-2">
          <Button disabled={page <= 1} onClick={() => setPage((p) => Math.max(1, p - 1))}>Prev</Button>
          <span className="py-2 px-2">Page {page}</span>
          <Button onClick={() => setPage((p) => p + 1)}>Next</Button>
        </div>
      </div>

      <Dialog open={open} onOpenChange={setOpen}>
        <DialogHeader><DialogTitle>Add Extension — Step {step} of 3</DialogTitle></DialogHeader>
        <div className="flex gap-1 mb-4">
          {[1, 2, 3].map((s) => <div key={s} className={`h-1 flex-1 rounded ${s <= step ? "bg-primary" : "bg-muted"}`} />)}
        </div>
        {step === 1 && (
          <div className="space-y-3">
            <div><Label>Extension *</Label><Input value={form.extension} onChange={(e) => setForm({ ...form, extension: e.target.value })} placeholder="1001" pattern="\d{2,7}" /></div>
            <div><Label>Display Name *</Label><Input value={form.effective_caller_id_name} onChange={(e) => setForm({ ...form, effective_caller_id_name: e.target.value })} placeholder="Alice Smith" /></div>
            <Button onClick={() => setStep(2)} className="w-full">Next: Caller ID</Button>
          </div>
        )}
        {step === 2 && (
          <div className="space-y-3">
            <div><Label>Caller ID Number</Label><Input value={form.effective_caller_id_number} onChange={(e) => setForm({ ...form, effective_caller_id_number: e.target.value })} placeholder="+15551234567" /></div>
            <div className="rounded border p-3 bg-muted/50 text-sm">
              <div className="font-medium">Forwarding (Advanced)</div>
              <div className="text-xs text-muted-foreground">Hidden by default — expand to configure forward_all / busy / no_answer</div>
              <details className="mt-2"><summary className="cursor-pointer text-sm">Show forwarding</summary>
                <div className="mt-2 space-y-2"><Input placeholder="Forward All" /><Input placeholder="Forward Busy" /><Input placeholder="Forward No Answer" /></div>
              </details>
            </div>
            <div className="flex gap-2"><Button variant="outline" onClick={() => setStep(1)} className="flex-1">Back</Button><Button onClick={() => setStep(3)} className="flex-1">Next: Review</Button></div>
          </div>
        )}
        {step === 3 && (
          <div className="space-y-3">
            <Card><CardContent className="pt-4 text-sm space-y-1"><div>Extension: <span className="font-mono">{form.extension}</span></div><div>Name: {form.effective_caller_id_name}</div><div>Number: {form.effective_caller_id_number || "-"}</div></CardContent></Card>
            <div className="flex gap-2"><Button variant="outline" onClick={() => setStep(2)} className="flex-1">Back</Button><Button onClick={create} className="flex-1">Create</Button></div>
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
