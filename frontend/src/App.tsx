import * as React from "react";
import { BrowserRouter, Routes, Route, Link, useLocation } from "react-router-dom";
import { ThemeProvider, ThemeToggle } from "./components/theme/ThemeProvider";
import { Dashboard } from "./routes/Dashboard";
import { Extensions } from "./routes/Extensions";
import { Gateways } from "./routes/Gateways";
import { Cdr } from "./routes/Cdr";

function Nav() {
  const [open, setOpen] = React.useState(false);
  const loc = useLocation();
  const items = [
    { to: "/", label: "Dashboard" },
    { to: "/extensions", label: "Extensions" },
    { to: "/gateways", label: "Gateways" },
    { to: "/cdr", label: "Reports" },
    { to: "/devices", label: "Devices" },
    { to: "/dialplans", label: "Call Routing" },
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
        </div>
        <div className="flex items-center gap-2">
          <ThemeToggle />
        </div>
      </header>
      <div className="flex">
        {/* Desktop sidebar */}
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
            <details>
              <summary className="cursor-pointer px-3 py-2 text-sm font-medium">Advanced</summary>
              <div className="pl-4 pt-2 flex flex-col gap-1">
                <Link to="/ivr" className="rounded px-3 py-2 text-sm hover:bg-muted">IVR Menus</Link>
                <Link to="/ring-groups" className="rounded px-3 py-2 text-sm hover:bg-muted">Ring Groups</Link>
                <Link to="/plugins" className="rounded px-3 py-2 text-sm hover:bg-muted">Plugins</Link>
              </div>
            </details>
          </div>
        </nav>

        {/* Mobile drawer */}
        {open && (
          <nav className="fixed inset-0 z-10 flex flex-col bg-background p-4 md:hidden" data-testid="mobile-drawer">
            <div className="flex justify-between items-center mb-4">
              <span className="font-bold">Menu</span>
              <button onClick={() => setOpen(false)} className="h-10 w-10 border rounded min-h-[44px] min-w-[44px]">✕</button>
            </div>
            {items.map((it) => (
              <Link
                key={it.to}
                to={it.to}
                onClick={() => setOpen(false)}
                className={`rounded px-3 py-3 text-base min-h-[44px] ${loc.pathname === it.to ? "bg-primary text-primary-foreground" : "hover:bg-muted"}`}
              >
                {it.label}
              </Link>
            ))}
            <div className="mt-4 border-t pt-4">
              <span className="px-3 py-2 text-sm font-medium">Advanced</span>
              <Link to="/ivr" onClick={() => setOpen(false)} className="rounded px-3 py-3 text-base hover:bg-muted">IVR Menus</Link>
            </div>
          </nav>
        )}

        <main className="flex-1 min-w-0">
          <Routes>
            <Route path="/" element={<Dashboard />} />
            <Route path="/extensions" element={<Extensions />} />
            <Route path="/gateways" element={<Gateways />} />
            <Route path="/cdr" element={<Cdr />} />
            <Route path="/devices" element={<div className="p-4">Devices placeholder</div>} />
            <Route path="/dialplans" element={<div className="p-4">Dialplans placeholder</div>} />
            <Route path="/ivr" element={<div className="p-4">IVR placeholder</div>} />
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
