import * as React from "react";
import { BrowserRouter, Routes, Route, Link, useLocation, useNavigate, Navigate } from "react-router-dom";
import { ThemeProvider, ThemeToggle } from "./components/theme/ThemeProvider";
import { Toaster } from "./lib/toast";
import { FsStatus } from "./components/FsStatus";
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
import { PluginDetail } from "./routes/PluginDetail";
import { SipProfiles } from "./routes/SipProfiles";
import { Login } from "./routes/Login";
import { Setup } from "./routes/Setup";
import { Bridges } from "./routes/Bridges";
import { Streams } from "./routes/Streams";
import { Phrases } from "./routes/Phrases";
import { PinNumbers } from "./routes/PinNumbers";
import { Destinations } from "./routes/Destinations";
import { CallBlock } from "./routes/CallBlock";
import { FollowMe } from "./routes/FollowMe";
import { MusicOnHold } from "./routes/MusicOnHold";
import { CallFlows } from "./routes/CallFlows";
import { Fax } from "./routes/Fax";
import { Conferences } from "./routes/Conferences";
import { Voicemails } from "./routes/Voicemails";
import { TimeConditions } from "./routes/TimeConditions";
import { RingGroups } from "./routes/RingGroups";
import { Fifo } from "./routes/Fifo";
import { CallCenters } from "./routes/CallCenters";
import { Vars } from "./routes/Vars";
import { NumberTranslations } from "./routes/NumberTranslations";
import { ConferenceProfiles } from "./routes/ConferenceProfiles";
import { ConferenceControls } from "./routes/ConferenceControls";
import { ConferenceCenters } from "./routes/ConferenceCenters";
import { AccessControls } from "./routes/AccessControls";
import { CallBroadcasts } from "./routes/CallBroadcasts";
import { EmergencyLogs } from "./routes/EmergencyLogs";
import { VoicemailGreetings } from "./routes/VoicemailGreetings";
import { FaxQueue } from "./routes/FaxQueue";
import { ExtensionSettings } from "./routes/ExtensionSettings";
import { EmailQueue } from "./routes/EmailQueue";
import { SofiaGlobalSettings } from "./routes/SofiaGlobalSettings";
import { FeatureCodes } from "./routes/FeatureCodes";
import { DatabaseTransactions } from "./routes/DatabaseTransactions";
import { EventGuard } from "./routes/EventGuard";
import { Modules } from "./routes/Modules";
import { ActiveCalls } from "./routes/ActiveCalls";
import { Registrations } from "./routes/Registrations";
import { OperatorPanel } from "./routes/OperatorPanel";
import { Logs } from "./routes/Logs";
import { DomainLimits } from "./routes/DomainLimits";

function RequireAuth({ children }: { children: React.ReactNode }) {
  const token = typeof window !== "undefined" ? localStorage.getItem("token") : null;
  const loc = useLocation();
  if (!token) {
    return <Navigate to={`/login?next=${encodeURIComponent(loc.pathname + loc.search)}`} replace />;
  }
  return <>{children}</>;
}

function Nav() {
  const [open, setOpen] = React.useState(false);
  const loc = useLocation();
  const nav = useNavigate();
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
          <FsStatus />
          <Link to="/domains" className="hidden md:inline text-sm hover:underline">Tenants</Link>
          <ThemeToggle />
          <button
            onClick={() => {
              localStorage.removeItem("token");
              nav("/login");
            }}
            className="hidden md:inline text-sm border rounded px-3 py-1 hover:bg-muted"
            data-testid="logout-btn"
          >
            Logout
          </button>
        </div>
      </header>
      <div className="flex">
        {/* Desktop sidebar — 6 primary items per spec, sticky + thin hover scrollbar */}
        <nav className="hidden md:flex w-56 shrink-0 flex-col border-r bg-background sticky top-14 h-[calc(100vh-3.5rem)] overflow-hidden">
          <div className="flex-1 overflow-y-auto custom-scrollbar p-4 flex flex-col gap-1">
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
                <Link to="/sip_profiles" className={`rounded px-3 py-2 text-sm ${loc.pathname === "/sip_profiles" ? "bg-muted" : "hover:bg-muted"}`}>SIP Profiles</Link>
                <Link to="/ring_groups" className={`rounded px-3 py-2 text-sm ${loc.pathname === "/ring_groups" ? "bg-muted" : "hover:bg-muted"}`}>Ring Groups</Link>
                <Link to="/time_conditions" className={`rounded px-3 py-2 text-sm ${loc.pathname === "/time_conditions" ? "bg-muted" : "hover:bg-muted"}`}>Time Conditions</Link>
                <Link to="/voicemails" className={`rounded px-3 py-2 text-sm ${loc.pathname === "/voicemails" ? "bg-muted" : "hover:bg-muted"}`}>Voicemails</Link>
                <Link to="/conferences" className={`rounded px-3 py-2 text-sm ${loc.pathname === "/conferences" ? "bg-muted" : "hover:bg-muted"}`}>Conferences</Link>
                <Link to="/conference_profiles" className={`rounded px-3 py-2 text-sm ${loc.pathname === "/conference_profiles" ? "bg-muted" : "hover:bg-muted"}`}>Conf Profiles</Link>
                <Link to="/conference_controls" className={`rounded px-3 py-2 text-sm ${loc.pathname === "/conference_controls" ? "bg-muted" : "hover:bg-muted"}`}>Conf Controls</Link>
                <Link to="/conference_centers" className={`rounded px-3 py-2 text-sm ${loc.pathname === "/conference_centers" ? "bg-muted" : "hover:bg-muted"}`}>Conf Centers</Link>
                <Link to="/access_controls" className={`rounded px-3 py-2 text-sm ${loc.pathname === "/access_controls" ? "bg-muted" : "hover:bg-muted"}`}>ACL</Link>
                <Link to="/call_block" className={`rounded px-3 py-2 text-sm ${loc.pathname === "/call_block" ? "bg-muted" : "hover:bg-muted"}`}>Block</Link>
                <Link to="/call_centers" className={`rounded px-3 py-2 text-sm ${loc.pathname === "/call_centers" ? "bg-muted" : "hover:bg-muted"}`}>Call Centers</Link>
                <Link to="/fifo" className={`rounded px-3 py-2 text-sm ${loc.pathname === "/fifo" ? "bg-muted" : "hover:bg-muted"}`}>Fifo</Link>
                <Link to="/bridges" className={`rounded px-3 py-2 text-sm ${loc.pathname === "/bridges" ? "bg-muted" : "hover:bg-muted"}`}>Bridges</Link>
                <Link to="/streams" className={`rounded px-3 py-2 text-sm ${loc.pathname === "/streams" ? "bg-muted" : "hover:bg-muted"}`}>Streams</Link>
                <Link to="/phrases" className={`rounded px-3 py-2 text-sm ${loc.pathname === "/phrases" ? "bg-muted" : "hover:bg-muted"}`}>Phrases</Link>
                <Link to="/pin_numbers" className={`rounded px-3 py-2 text-sm ${loc.pathname === "/pin_numbers" ? "bg-muted" : "hover:bg-muted"}`}>PINs</Link>
                <Link to="/destinations" className={`rounded px-3 py-2 text-sm ${loc.pathname === "/destinations" ? "bg-muted" : "hover:bg-muted"}`}>Destinations</Link>
                <Link to="/follow_me" className={`rounded px-3 py-2 text-sm ${loc.pathname === "/follow_me" ? "bg-muted" : "hover:bg-muted"}`}>FollowMe</Link>
                <Link to="/music_on_hold" className={`rounded px-3 py-2 text-sm ${loc.pathname === "/music_on_hold" ? "bg-muted" : "hover:bg-muted"}`}>MOH</Link>
                <Link to="/call_flows" className={`rounded px-3 py-2 text-sm ${loc.pathname === "/call_flows" ? "bg-muted" : "hover:bg-muted"}`}>Call Flows</Link>
                <Link to="/fax" className={`rounded px-3 py-2 text-sm ${loc.pathname === "/fax" ? "bg-muted" : "hover:bg-muted"}`}>Fax</Link>
                <Link to="/fax_queue" className={`rounded px-3 py-2 text-sm ${loc.pathname === "/fax_queue" ? "bg-muted" : "hover:bg-muted"}`}>Fax Queue</Link>
                <Link to="/vars" className={`rounded px-3 py-2 text-sm ${loc.pathname === "/vars" ? "bg-muted" : "hover:bg-muted"}`}>Vars</Link>
                <Link to="/number_translations" className={`rounded px-3 py-2 text-sm ${loc.pathname === "/number_translations" ? "bg-muted" : "hover:bg-muted"}`}>Number Trans</Link>
                <Link to="/call_broadcasts" className={`rounded px-3 py-2 text-sm ${loc.pathname === "/call_broadcasts" ? "bg-muted" : "hover:bg-muted"}`}>Broadcasts</Link>
                <Link to="/emergency_logs" className={`rounded px-3 py-2 text-sm ${loc.pathname === "/emergency_logs" ? "bg-muted" : "hover:bg-muted"}`}>Emergency</Link>
                <Link to="/voicemail_greetings" className={`rounded px-3 py-2 text-sm ${loc.pathname === "/voicemail_greetings" ? "bg-muted" : "hover:bg-muted"}`}>VM Greetings</Link>
                <Link to="/extension_settings" className={`rounded px-3 py-2 text-sm ${loc.pathname === "/extension_settings" ? "bg-muted" : "hover:bg-muted"}`}>Ext Settings</Link>
                <Link to="/email_queue" className={`rounded px-3 py-2 text-sm ${loc.pathname === "/email_queue" ? "bg-muted" : "hover:bg-muted"}`}>Email Queue</Link>
                <Link to="/sofia_global_settings" className={`rounded px-3 py-2 text-sm ${loc.pathname === "/sofia_global_settings" ? "bg-muted" : "hover:bg-muted"}`}>Sofia Global</Link>
                <Link to="/feature_codes" className={`rounded px-3 py-2 text-sm ${loc.pathname === "/feature_codes" ? "bg-muted" : "hover:bg-muted"}`}>Feature Codes</Link>
                <Link to="/database_transactions" className={`rounded px-3 py-2 text-sm ${loc.pathname === "/database_transactions" ? "bg-muted" : "hover:bg-muted"}`}>DB Transactions</Link>
                <Link to="/event_guard" className={`rounded px-3 py-2 text-sm ${loc.pathname === "/event_guard" ? "bg-muted" : "hover:bg-muted"}`}>Event Guard</Link>
                <Link to="/modules" className={`rounded px-3 py-2 text-sm ${loc.pathname === "/modules" ? "bg-muted" : "hover:bg-muted"}`}>Modules</Link>
                <Link to="/active_calls" className={`rounded px-3 py-2 text-sm ${loc.pathname === "/active_calls" ? "bg-muted" : "hover:bg-muted"}`}>Active Calls</Link>
                <Link to="/registrations" className={`rounded px-3 py-2 text-sm ${loc.pathname === "/registrations" ? "bg-muted" : "hover:bg-muted"}`}>Registrations</Link>
                <Link to="/operator_panel" className={`rounded px-3 py-2 text-sm ${loc.pathname === "/operator_panel" ? "bg-muted" : "hover:bg-muted"}`}>Operator</Link>
                <Link to="/logs" className={`rounded px-3 py-2 text-sm ${loc.pathname === "/logs" ? "bg-muted" : "hover:bg-muted"}`}>Logs</Link>
                <Link to="/domain_limits" className={`rounded px-3 py-2 text-sm ${loc.pathname === "/domain_limits" ? "bg-muted" : "hover:bg-muted"}`}>Limits</Link>
                <Link to="/users" className={`rounded px-3 py-2 text-sm ${loc.pathname === "/users" ? "bg-muted" : "hover:bg-muted"}`}>Users & Groups</Link>
                <Link to="/domains" className={`rounded px-3 py-2 text-sm ${loc.pathname === "/domains" ? "bg-muted" : "hover:bg-muted"}`}>Tenants</Link>
                <Link to="/plugins" className={`rounded px-3 py-2 text-sm ${loc.pathname === "/plugins" ? "bg-muted" : "hover:bg-muted"}`}>Plugins</Link>
              </div>
            </details>
          </div>
          </div>
          <div className="shrink-0 border-t p-4 text-xs text-muted-foreground">
            <div>PBX v1.0.0</div>
            <div>REST /api/v1</div>
          </div>
        </nav>

        {/* Mobile drawer — full scroll, no nested 50vh box */}
        {open && (
          <nav className="fixed inset-0 z-30 flex flex-col bg-background p-4 md:hidden overflow-y-auto custom-scrollbar" data-testid="mobile-drawer">
            <div className="flex justify-between items-center mb-4 shrink-0">
              <span className="font-bold">Menu</span>
              <button onClick={() => setOpen(false)} className="h-10 w-10 border rounded min-h-[44px] min-w-[44px]">✕</button>
            </div>
            <div className="flex flex-col gap-1 shrink-0">
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
                <Link to="/sip_profiles" onClick={() => setOpen(false)} className="rounded px-3 py-3 text-base hover:bg-muted border">SIP Profiles</Link>
                <Link to="/ring_groups" onClick={() => setOpen(false)} className="rounded px-3 py-3 text-base hover:bg-muted border">Ring Groups</Link>
                <Link to="/time_conditions" onClick={() => setOpen(false)} className="rounded px-3 py-3 text-base hover:bg-muted border">Time Conditions</Link>
                <Link to="/voicemails" onClick={() => setOpen(false)} className="rounded px-3 py-3 text-base hover:bg-muted border">Voicemails</Link>
                <Link to="/conferences" onClick={() => setOpen(false)} className="rounded px-3 py-3 text-base hover:bg-muted border">Conferences</Link>
                <Link to="/call_flows" onClick={() => setOpen(false)} className="rounded px-3 py-3 text-base hover:bg-muted border">Call Flows</Link>
                <Link to="/music_on_hold" onClick={() => setOpen(false)} className="rounded px-3 py-3 text-base hover:bg-muted border">MOH</Link>
                <Link to="/fifo" onClick={() => setOpen(false)} className="rounded px-3 py-3 text-base hover:bg-muted border">Fifo</Link>
                <Link to="/call_centers" onClick={() => setOpen(false)} className="rounded px-3 py-3 text-base hover:bg-muted border">Call Centers</Link>
                <Link to="/access_controls" onClick={() => setOpen(false)} className="rounded px-3 py-3 text-base hover:bg-muted border">ACL</Link>
                <Link to="/active_calls" onClick={() => setOpen(false)} className="rounded px-3 py-3 text-base hover:bg-muted border">Active Calls</Link>
                <Link to="/operator_panel" onClick={() => setOpen(false)} className="rounded px-3 py-3 text-base hover:bg-muted border">Operator</Link>
                <Link to="/logs" onClick={() => setOpen(false)} className="rounded px-3 py-3 text-base hover:bg-muted border">Logs</Link>
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
            <Route path="/ring_groups" element={<RingGroups />} />
            <Route path="/time_conditions" element={<TimeConditions />} />
            <Route path="/voicemails" element={<Voicemails />} />
            <Route path="/conferences" element={<Conferences />} />
            <Route path="/fax" element={<Fax />} />
            <Route path="/call_flows" element={<CallFlows />} />
            <Route path="/music_on_hold" element={<MusicOnHold />} />
            <Route path="/follow_me" element={<FollowMe />} />
            <Route path="/call_block" element={<CallBlock />} />
            <Route path="/pin_numbers" element={<PinNumbers />} />
            <Route path="/call_centers" element={<CallCenters />} />
            <Route path="/fifo" element={<Fifo />} />
            <Route path="/bridges" element={<Bridges />} />
            <Route path="/streams" element={<Streams />} />
            <Route path="/phrases" element={<Phrases />} />
            <Route path="/conference_profiles" element={<ConferenceProfiles />} />
            <Route path="/conference_controls" element={<ConferenceControls />} />
            <Route path="/conference_centers" element={<ConferenceCenters />} />
            <Route path="/number_translations" element={<NumberTranslations />} />
            <Route path="/access_controls" element={<AccessControls />} />
            <Route path="/call_broadcasts" element={<CallBroadcasts />} />
            <Route path="/emergency_logs" element={<EmergencyLogs />} />
            <Route path="/voicemail_greetings" element={<VoicemailGreetings />} />
            <Route path="/fax_queue" element={<FaxQueue />} />
            <Route path="/extension_settings" element={<ExtensionSettings />} />
            <Route path="/email_queue" element={<EmailQueue />} />
            <Route path="/sofia_global_settings" element={<SofiaGlobalSettings />} />
            <Route path="/feature_codes" element={<FeatureCodes />} />
            <Route path="/database_transactions" element={<DatabaseTransactions />} />
            <Route path="/event_guard" element={<EventGuard />} />
            <Route path="/modules" element={<Modules />} />
            <Route path="/active_calls" element={<ActiveCalls />} />
            <Route path="/registrations" element={<Registrations />} />
            <Route path="/operator_panel" element={<OperatorPanel />} />
            <Route path="/logs" element={<Logs />} />
            <Route path="/domain_limits" element={<DomainLimits />} />
            <Route path="/vars" element={<Vars />} />
            <Route path="/destinations" element={<Destinations />} />
            <Route path="/users" element={<Users />} />
            <Route path="/domains" element={<Domains />} />
            <Route path="/sip_profiles" element={<SipProfiles />} />
            <Route path="/plugins" element={<Plugins />} />
            <Route path="/plugins/:name" element={<PluginDetail />} />
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
      <Toaster />
      <BrowserRouter>
        <Routes>
          <Route path="/login" element={<Login />} />
          <Route path="/setup" element={<Setup />} />
          <Route
            path="/*"
            element={
              <RequireAuth>
                <Nav />
              </RequireAuth>
            }
          />
        </Routes>
      </BrowserRouter>
    </ThemeProvider>
  );
}
