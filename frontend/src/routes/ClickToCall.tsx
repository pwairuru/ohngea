import * as React from "react";
import { apiFetch } from "../lib/api";
import { Card, CardContent, CardHeader, CardTitle } from "../components/ui/card";
import { PageHeader } from "../components/shell/PageHeader";
import { Button } from "../components/ui/button";
import { Label } from "../components/ui/label";
import { Input } from "../components/ui/input";
import { toast, toastError } from "../lib/toast";
import { useAsyncAction } from "../lib/useAsyncAction";

export function ClickToCall() {
  const [form, setForm] = React.useState({ extension: "1001", number: "1002" });
  const [result, setResult] = React.useState<any | null>(null);

  const { run: originate, pending } = useAsyncAction(async () => {
    try {
      const res = await apiFetch("/click_to_call", {
        method: "POST",
        body: JSON.stringify({ extension: form.extension, number: form.number }),
      });
      setResult(res);
      toast("Originate sent", { variant: "success" });
    } catch (err) {
      toastError(err);
      setResult({ error: String(err) });
    }
  });

  return (
    <>
      <PageHeader title="Click to Call" description="Originate a call between extension and destination via ESL" />
      <div className="grid gap-4 md:grid-cols-2">
        <Card>
          <CardHeader><CardTitle className="text-sm">Originate</CardTitle></CardHeader>
          <CardContent className="space-y-4">
            <div>
              <Label>Extension (source) *</Label>
              <Input value={form.extension} onChange={(e) => setForm({ ...form, extension: e.target.value })} placeholder="1001" data-testid="ctc-ext" />
            </div>
            <div>
              <Label>Destination number *</Label>
              <Input value={form.number} onChange={(e) => setForm({ ...form, number: e.target.value })} placeholder="1002" data-testid="ctc-dest" />
            </div>
            <Button onClick={originate} disabled={pending || !form.extension || !form.number} data-testid="ctc-submit">
              {pending ? "Originating…" : "Call"}
            </Button>
            <p className="text-xs text-muted-foreground">POST /api/v1/click_to_call — delegates to ESL <code>originate user/… &amp;bridge</code> or queues if FS offline.</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader><CardTitle className="text-sm">Result</CardTitle></CardHeader>
          <CardContent>
            {result ? (
              <pre className="rounded bg-muted p-3 text-xs break-all whitespace-pre-wrap">{JSON.stringify(result, null, 2)}</pre>
            ) : (
              <div className="text-sm text-muted-foreground">No originate yet.</div>
            )}
          </CardContent>
        </Card>
      </div>
    </>
  );
}
