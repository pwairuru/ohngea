export const API_BASE = "/api/v1";

export function authHeaders(): HeadersInit {
  try {
    const token = typeof window !== "undefined" ? localStorage.getItem("token") : null;
    return token ? { Authorization: `Bearer ${token}` } : {};
  } catch {
    return {};
  }
}

export class ApiError extends Error {
  status: number;
  body: string;
  constructor(status: number, body: string) {
    super(`${status} ${body}`);
    this.status = status;
    this.body = body;
  }
}

export async function apiFetch(path: string, opts: RequestInit = {}) {
  const headers: Record<string, string> = {
    "Content-Type": "application/json",
    Accept: "application/vnd.pbx.v1+json",
    ...(authHeaders() as Record<string, string>),
    ...((opts.headers as Record<string, string>) || {}),
  };
  const res = await fetch(`${API_BASE}${path}`, { credentials: "include", ...opts, headers });
  if (res.status === 401) {
    try {
      localStorage.removeItem("token");
    } catch { /* noop */ }
    if (typeof window !== "undefined" && !window.location.pathname.startsWith("/login")) {
      const next = encodeURIComponent(window.location.pathname + window.location.search);
      window.location.href = `/login?next=${next}`;
    }
    const txt = await res.text();
    throw new ApiError(res.status, txt);
  }
  if (!res.ok) {
    const txt = await res.text();
    throw new ApiError(res.status, txt);
  }
  // handle 204
  if (res.status === 204) return null;
  const ct = res.headers.get("content-type") || "";
  if (ct.includes("application/json")) return res.json();
  return res.text();
}

export async function login(username: string, password: string, domain: string) {
  // Canonical versioned endpoint only — /auth/* 307-redirects to /api/v1/auth/*
  // credentials:include so httpOnly `access_token` cookie is stored; token also
  // returned for Bearer compat (kept in localStorage by caller).
  const res = await fetch(`${API_BASE}/auth/login`, {
    method: "POST",
    credentials: "include",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ username, password, domain }),
  });
  if (!res.ok) throw new Error("login failed");
  return res.json();
}
