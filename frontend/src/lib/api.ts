export const API_BASE = "/api/v1";

export function authHeaders(): HeadersInit {
  const token = localStorage.getItem("token");
  return token ? { Authorization: `Bearer ${token}` } : {};
}

export async function apiFetch(path: string, opts: RequestInit = {}) {
  const headers: Record<string, string> = {
    "Content-Type": "application/json",
    Accept: "application/vnd.pbx.v1+json",
    ...(authHeaders() as Record<string, string>),
    ...((opts.headers as Record<string, string>) || {}),
  };
  const res = await fetch(`${API_BASE}${path}`, { ...opts, headers });
  if (!res.ok) {
    const txt = await res.text();
    if (res.status === 401) {
      // clear stale token and redirect to login if not already there
      const isLogin = window.location.pathname === "/login" || window.location.pathname === "/setup";
      if (!isLogin) {
        localStorage.removeItem("token");
        // allow caller to handle, but also redirect
        // use location to avoid React Router hook outside component
        const next = encodeURIComponent(window.location.pathname + window.location.search);
        window.location.href = `/login?next=${next}`;
      }
    }
    throw new Error(`${res.status} ${txt}`);
  }
  // handle 204
  if (res.status === 204) return null;
  const ct = res.headers.get("content-type") || "";
  if (ct.includes("application/json")) return res.json();
  return res.text();
}

export async function login(username: string, password: string, domain: string) {
  // try versioned then fallback
  const res = await fetch("/auth/login", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ username, password, domain }),
  });
  if (!res.ok) {
    // try /api/v1/auth/login
    const r2 = await fetch("/api/v1/auth/login", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ username, password, domain }),
    });
    if (!r2.ok) throw new Error("login failed");
    return r2.json();
  }
  return res.json();
}
