import * as React from "react";
import { useNavigate, Link } from "react-router-dom";
import { Card, CardContent, CardHeader, CardTitle } from "../components/ui/card";
import { Button } from "../components/ui/button";
import { Input } from "../components/ui/input";
import { Label } from "../components/ui/label";

export function Setup() {
  const nav = useNavigate();
  const [form, setForm] = React.useState({ domain_name: "", admin_username: "admin", admin_password: "admin" });
  const [error, setError] = React.useState("");
  const [info, setInfo] = React.useState("");
  const [checking, setChecking] = React.useState(true);

  React.useEffect(() => {
    fetch("/api/v1/domains/bootstrap/status")
      .then((r) => r.json())
      .then((j) => {
        if (!j.needs_setup) {
          setInfo("System already bootstrapped — please sign in");
        }
      })
      .catch(() => {})
      .finally(() => setChecking(false));
  }, []);

  const submit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError("");
    setInfo("");
    try {
      const r = await fetch("/api/v1/domains/bootstrap", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(form),
      });
      if (!r.ok) {
        const txt = await r.text();
        throw new Error(txt || "bootstrap failed");
      }
      const data = await r.json();
      setInfo(`Domain ${data.domain_name} created — you can now sign in as ${form.admin_username}`);
      // store domain for login convenience
      localStorage.setItem("domain", form.domain_name);
      setTimeout(() => nav("/login"), 1500);
    } catch (err: any) {
      setError(err.message);
    }
  };

  if (checking) return <div className="p-8 text-center">Checking...</div>;

  return (
    <div className="min-h-screen flex items-center justify-center bg-muted/20 p-4">
      <Card className="w-full max-w-md">
        <CardHeader>
          <CardTitle>Initial Setup</CardTitle>
          <p className="text-sm text-muted-foreground">Create the first tenant domain and admin user. Only allowed when no domains exist.</p>
        </CardHeader>
        <CardContent>
          <form onSubmit={submit} className="space-y-4">
            <div>
              <Label>Domain name *</Label>
              <Input value={form.domain_name} onChange={(e) => setForm({ ...form, domain_name: e.target.value })} placeholder="pbx.example.com" required />
            </div>
            <div>
              <Label>Admin username</Label>
              <Input value={form.admin_username} onChange={(e) => setForm({ ...form, admin_username: e.target.value })} />
            </div>
            <div>
              <Label>Admin password</Label>
              <Input type="password" value={form.admin_password} onChange={(e) => setForm({ ...form, admin_password: e.target.value })} />
            </div>
            {error && <div className="rounded border bg-destructive/10 p-3 text-sm text-destructive">{error}</div>}
            {info && <div className="rounded border bg-green-50 p-3 text-sm text-green-800">{info}</div>}
            <Button type="submit" className="w-full">Create domain</Button>
            <div className="text-center text-sm">
              <Link to="/login" className="text-primary hover:underline">Already set up? Sign in</Link>
            </div>
          </form>
        </CardContent>
      </Card>
    </div>
  );
}
