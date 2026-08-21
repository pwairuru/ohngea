import * as React from "react";
import { BrowserRouter, Routes, Route, Link, useLocation } from "react-router-dom";
import { ThemeProvider, ThemeToggle } from "./components/theme/ThemeProvider";
import { Dashboard } from "./routes/Dashboard";
import { Extensions } from "./routes/Extensions";
import { Gateways } from "./routes/Gateways";
import { Cdr } from "./routes/Cdr";
import { Dialplans } from "./routes/Dialplans";
import { Devices } from "./routes/Devices";
import { IvrMenus } from "./routes/IvrMenus";
import { Users } from "./routes/Users";
import { Domains } from "./routes/Domains";
import { Plugins } from "./routes/Plugins";

function Nav() {
  const [open, setOpen] = React.useState(false);
  const loc = useLocation();
  const items = [
    { to: "/", label: "Dashboard" },
    { to: "/extensions", label: "Extensions" },
    { to: "/dialplans", label: "Call Routing" },
    { to: "/gateways", label: "Gateways" },
    { to: "/devices", label: "Devices" },
    { to: "/cdr", label: "Reports" },
  ];
  return (
    <>
      <header className="sticky top-0 z-20 flex h-14 items-center justify-between border-b bg-background px-4 md:px-6">
        <div className="flex items-center gap-2">
          <button
            data-testid="mobile-nav-toggle"
            className="md:hidden inline-flex h-10 w-10 items-center justify-center rounded-md border min-h-[44px] min-w-[44px]"
            onClick={() => setOpen((v) => !v)}
            aria-label="Toggle navigation"
          >
            ☰
          </button>
          <Link to="/" className="font-bold text-lg">PBX</Link>
          <span className="hidden sm:inline text-xs text-muted-foreground ml-2">Modern</span>
        </div>
        <div className="flex items-center gap-2">
          <Link to="/domains" className="hidden md:inline text-sm hover:underline">Tenants</Link>
          <ThemeToggle />
        </div>
      </header>
      <div className="flex">
        {/* Desktop sidebar — 6 primary items per spec */}
        <nav className="hidden md:flex w-56 shrink-0 flex-col border-r p-4 gap-1">
          {items.map((it) => (
            <Link
              key={it.to}
              to={it.to}
              className={`rounded px-3 py-2 text-sm ${loc.pathname === it.to ? "bg-primary text-primary-foreground" : "hover:bg-muted"}`}
            >
              {it.label}
            </Link>
          ))}
          <div className="mt-4 border-t pt-4">
            <details open>
              <summary className="cursor-pointer px-3 py-2 text-sm font-medium">Advanced</summary>
              <div className="pl-2 pt-2 flex flex-col gap-1">
                <Link to="/ivr" className={`rounded px-3 py-2 text-sm ${loc.pathname === "/ivr" ? "bg-muted font-medium" : "hover:bg-muted"}`}>IVR Menus</Link>
                <Link to="/users" className={`rounded px-3 py-2 text-sm ${loc.pathname === "/users" ? "bg-muted" : "hover:bg-muted"}`}>Users & Groups</Link>
                <Link to="/domains" className={`rounded px-3 py-2 text-sm ${loc.pathname === "/domains" ? "bg-muted" : "hover:bg-muted"}`}>Tenants</Link>
                <Link to="/plugins" className={`rounded px-3 py-2 text-sm ${loc.pathname === "/plugins" ? "bg-muted" : "hover:bg-muted"}`}>Plugins</Link>
              </div>
            </details>
          </div>
          <div className="mt-auto pt-4 text-xs text-muted-foreground">
            <div>PBX v1.0.0</div>
            <div>REST /api/v1</div>
          </div>
        </nav>

        {/* Mobile drawer */}
        {open && (
          <nav className="fixed inset-0 z-30 flex flex-col bg-background p-4 md:hidden" data-testid="mobile-drawer">
            <div className="flex justify-between items-center mb-4">
              <span className="font-bold">Menu</span>
              <button onClick={() => setOpen(false)} className="h-10 w-10 border rounded min-h-[44px] min-w-[44px]">✕</button>
            </div>
            <div className="flex flex-col gap-1">
              {items.map((it) => (
                <Link
                  key={it.to}
                  to={it.to}
                  onClick={() => setOpen(false)}
                  className={`rounded px-3 py-3 text-base min-h-[44px] ${loc.pathname === it.to ? "bg-primary text-primary-foreground" : "hover:bg-muted border"}`}
                >
                  {it.label}
                </Link>
              ))}
            </div>
            <div className="mt-4 border-t pt-4">
              <span className="px-3 py-2 text-sm font-medium">Advanced</span>
              <div className="flex flex-col gap-1 mt-1">
                <Link to="/ivr" onClick={() => setOpen(false)} className="rounded px-3 py-3 text-base hover:bg-muted border">IVR Menus</Link>
                <Link to="/users" onClick={() => setOpen(false)} className="rounded px-3 py-3 text-base hover:bg-muted border">Users & Groups</Link>
                <Link to="/domains" onClick={() => setOpen(false)} className="rounded px-3 py-3 text-base hover:bg-muted border">Tenants</Link>
                <Link to="/plugins" onClick={() => setOpen(false)} className="rounded px-3 py-3 text-base hover:bg-muted border">Plugins</Link>
              </div>
            </div>
          </nav>
        )}

        <main className="flex-1 min-w-0 bg-muted/20 min-h-[calc(100vh-3.5rem)]">
          <Routes>
            <Route path="/" element={<Dashboard />} />
            <Route path="/extensions" element={<Extensions />} />
            <Route path="/dialplans" element={<Dialplans />} />
            <Route path="/gateways" element={<Gateways />} />
            <Route path="/cdr" element={<Cdr />} />
            <Route path="/devices" element={<Devices />} />
            <Route path="/ivr" element={<IvrMenus />} />
            <Route path="/users" element={<Users />} />
            <Route path="/domains" element={<Domains />} />
            <Route path="/plugins" element={<Plugins />} />
            <Route path="*" element={<div className="p-4">Not found</div>} />
          </Routes>
        </main>
      </div>
    </>
  );
}

export default function App() {
  return (
    <ThemeProvider>
      <BrowserRouter>
        <Nav />
      </BrowserRouter>
    </ThemeProvider>
  );
}
