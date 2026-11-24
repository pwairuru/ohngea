import * as React from "react";
import { apiFetch } from "../lib/api";
import { PageHeader } from "../components/shell/PageHeader";
import { Button } from "../components/ui/button";
import { Label } from "../components/ui/label";
import { Input } from "../components/ui/input";
import { Card, CardContent, CardHeader, CardTitle } from "../components/ui/card";
import { toast, toastError } from "../lib/toast";
import { useAsyncAction } from "../lib/useAsyncAction";

export function ClickToCall() {
  const [form, setForm] = React.useState({ extension: "1001", number: "" });
  const [result, setResult] = React.useState("");

  const { run: originate, pending } = useAsyncAction(async () => {
    try {
      const res: any = await apiFetch("/click_to_call", {
        method: "POST",
        body: JSON.stringify({ extension: form.extension, number: form.number }),
      });
      setResult(res?.result ?? res?.status ?? "originated");
      toast(`Call ${res?.status ?? "originated"}`, { variant: "success" });
    } catch (err: any) {
      setResult(err?.message ?? "failed");
      toastError(err);
    }
  });

  return (
    <>
      <PageHeader title="Click to Call" description="Originate a call between your extension and a destination" />
      <Card>
        <CardHeader><CardTitle>Originate</CardTitle></CardHeader>
        <CardContent className="space-y-3 max-w-md">
          <div><Label>From extension *</Label><Input value={form.extension} onChange={(e) => setForm({ ...form, extension: e.target.value })} placeholder="1001" /></div>
          <div><Label>Destination number *</Label><Input value={form.number} onChange={(e) => setForm({ ...form, number: e.target.value })} placeholder="1002" /></div>
          <Button onClick={originate} disabled={pending || !form.number}>{pending ? "Originating…" : "Call Now"}</Button>
          {result && <pre className="text-xs bg-muted p-2 rounded overflow-auto max-h-40">{result}</pre>}
        </CardContent>
      </Card>
    </>
  );
}
