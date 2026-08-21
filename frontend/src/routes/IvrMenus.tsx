import * as React from "react";
import { apiFetch } from "../lib/api";
import { Card, CardContent, CardHeader, CardTitle } from "../components/ui/card";
import { Button } from "../components/ui/button";
import { Input } from "../components/ui/input";
import { Label } from "../components/ui/label";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "../components/ui/table";
import { Dialog, DialogHeader, DialogTitle } from "../components/ui/dialog";

export function IvrMenus() {
  const [data, setData] = React.useState<any[]>([]);
  const [open, setOpen] = React.useState(false);
  const [form, setForm] = React.useState({ name: "main-ivr", extension: "5000", options: "1:transfer:1000 XML $${domain},2:transfer:1001 XML $${domain}" });

  const load = () => apiFetch("/ivr_menus").then((r) => setData(r.data || [])).catch(() => {});
  React.useEffect(() => { load(); }, []);

  const create = async () => {
    try {
      const opts = form.options.split(",").map((s) => { const [digits, action, param] = s.split(":"); return { digits, action, param }; });
      await apiFetch("/ivr_menus", { method: "POST", body: JSON.stringify({ name: form.name, extension: form.extension, options: opts }) });
      setOpen(false); load();
    } catch (e: any) { alert(e.message); }
  };

  return (
    <div className="p-4 space-y-4">
      <div className="flex justify-between items-center"><h1 className="text-2xl font-bold">IVR Menus</h1><Button onClick={() => setOpen(true)}>Add IVR</Button></div>
      <Card>
        <CardHeader><CardTitle>IVR Builder — XML via lxml</CardTitle></CardHeader>
        <CardContent>
          <div className="hidden md:block">
            <Table>
              <TableHeader><TableRow><TableHead>Name</TableHead><TableHead>Extension</TableHead></TableRow></TableHeader>
              <TableBody>
                {data.map((m) => <TableRow key={m.ivr_menu_uuid}><TableCell>{m.name}</TableCell><TableCell className="font-mono">{m.extension || "-"}</TableCell></TableRow>)}
                {data.length===0 && <TableRow><TableCell colSpan={2} className="text-center text-muted-foreground py-8">No IVR menus</TableCell></TableRow>}
              </TableBody>
            </Table>
          </div>
          <div className="md:hidden grid gap-3">
            {data.map((m) => <Card key={m.ivr_menu_uuid} className="p-3"><div className="font-bold">{m.name}</div><div className="text-xs font-mono">{m.extension}</div></Card>)}
            {data.length===0 && <div className="text-center text-muted-foreground py-4">No IVR</div>}
          </div>
        </CardContent>
      </Card>
      <Dialog open={open} onOpenChange={setOpen}>
        <DialogHeader><DialogTitle>Add IVR</DialogTitle></DialogHeader>
        <div className="space-y-3">
          <div><Label>Name</Label><Input value={form.name} onChange={(e) => setForm({ ...form, name: e.target.value })} /></div>
          <div><Label>Extension</Label><Input value={form.extension} onChange={(e) => setForm({ ...form, extension: e.target.value })} /></div>
          <div><Label>Options (digits:action:param comma separated)</Label><Input value={form.options} onChange={(e) => setForm({ ...form, options: e.target.value })} /></div>
          <Button onClick={create} className="w-full">Create</Button>
        </div>
      </Dialog>
    </div>
  );
}
