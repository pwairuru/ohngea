import * as React from "react";
import { useNavigate, Link } from "react-router-dom";
import { Card, CardContent, CardHeader, CardTitle } from "../components/ui/card";
import { Button } from "../components/ui/button";
import { Input } from "../components/ui/input";
import { Label } from "../components/ui/label";

export function Login() {
  const nav = useNavigate();
  const [form, setForm] = React.useState({ username: "admin", password: "admin", domain: "" });
  const [error, setError] = React.useState("");
  const [checking, setChecking] = React.useState(true);

  React.useEffect(() => {
    // if already logged in with token, try to verify and redirect to /
    const token = localStorage.getItem("token");
    if (token) {
      fetch("/auth/me", { headers: { Authorization: `Bearer ${token}` } })
        .then((r) => {
          if (r.ok) nav("/");
        })
        .catch(() => {});
    }
    // check if bootstrap needed
    fetch("/api/v1/domains/bootstrap/status")
      .then((r) => r.json())
      .then((j) => {
        if (j.needs_setup) nav("/setup");
      })
      .catch(() => {})
      .finally(() => setChecking(false));
  }, [nav]);

  const submit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError("");
    try {
      const r = await fetch("/auth/login", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(form),
      });
      let data: any;
      if (!r.ok) {
        const r2 = await fetch("/api/v1/auth/login", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify(form),
        });
        if (!r2.ok) {
          const txt = await r2.text();
          throw new Error(txt || "login failed");
        }
        data = await r2.json();
      } else {
        data = await r.json();
      }
      const token = data.access_token;
      if (!token) throw new Error("no token in response");
      localStorage.setItem("token", token);
      // also store domain for convenience
      if (form.domain) localStorage.setItem("domain", form.domain);
      nav("/");
    } catch (err: any) {
      setError(err.message || "Login failed");
    }
  };

  if (checking) return <div className="p-8 text-center text-muted-foreground">Checking setup...</div>;

  return (
    <div className="min-h-screen flex items-center justify-center bg-muted/20 p-4">
      <Card className="w-full max-w-md">
        <CardHeader>
          <CardTitle className="text-2xl">Sign in to PBX</CardTitle>
          <p className="text-sm text-muted-foreground">Enter your credentials and domain to continue</p>
        </CardHeader>
        <CardContent>
          <form onSubmit={submit} className="space-y-4">
            <div>
              <Label>Domain *</Label>
              <Input
                value={form.domain}
                onChange={(e) => setForm({ ...form, domain: e.target.value })}
                placeholder="example.com"
                required
                data-testid="login-domain"
              />
              <p className="text-xs text-muted-foreground mt-1">Tenant domain name (e.g. pbx.example.com)</p>
            </div>
            <div>
              <Label>Username *</Label>
              <Input value={form.username} onChange={(e) => setForm({ ...form, username: e.target.value })} placeholder="admin" required data-testid="login-username" />
            </div>
            <div>
              <Label>Password *</Label>
              <Input type="password" value={form.password} onChange={(e) => setForm({ ...form, password: e.target.value })} required data-testid="login-password" />
            </div>
            {error && <div className="rounded border border-destructive/50 bg-destructive/10 p-3 text-sm text-destructive" data-testid="login-error">{error}</div>}
            <Button type="submit" className="w-full" data-testid="login-submit">Sign in</Button>
            <div className="text-center text-sm">
              <Link to="/setup" className="text-primary hover:underline">First time? Create initial domain</Link>
            </div>
          </form>
        </CardContent>
      </Card>
    </div>
  );
}
