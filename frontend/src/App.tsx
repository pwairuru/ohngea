import * as React from "react";
import { BrowserRouter, Routes, Route, Link, useLocation, useNavigate, Navigate } from "react-router-dom";
import { ThemeProvider, ThemeToggle } from "./components/theme/ThemeProvider";
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
import { Apps } from "./routes/Apps";
import { WebPhoneProvider } from "./context/WebPhoneContext";
import { WebPhoneButton } from "./components/WebPhone/WebPhoneButton";
import { WebPhonePopup } from "./components/WebPhone/WebPhonePopup";
import { useEnabledApps } from "./lib/apps";

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
  const [webPhoneOpen, setWebPhoneOpen] = React.useState(false);
  const loc = useLocation();
  const nav = useNavigate();
  const { isEnabled } = useEnabledApps();
  const items = [
    { to: "/", label: "Dashboard" },
    { to: "/extensions", label: "Extensions" },
    { to: "/dialplans", label: "Call Routing" },
    { to: "/gateways", label: "Gateways" },
    { to: "/devices", label: "Devices" },
    { to: "/cdr", label: "Reports" },
  ];
  const desktopAdvanced: { to: string; label: string; app: string }[] = [
    { to: "/ivr", label: "IVR Menus", app: "ivr_menus" },
    { to: "/sip_profiles", label: "SIP Profiles", app: "sip_profiles" },
    { to: "/ring_groups", label: "Ring Groups", app: "ring_groups" },
    { to: "/time_conditions", label: "Time Conditions", app: "time_conditions" },
    { to: "/voicemails", label: "Voicemails", app: "voicemails" },
    { to: "/conferences", label: "Conferences", app: "conferences" },
    { to: "/conference_profiles", label: "Conf Profiles", app: "conference_profiles" },
    { to: "/conference_controls", label: "Conf Controls", app: "conference_controls" },
    { to: "/conference_centers", label: "Conf Centers", app: "conference_centers" },
    { to: "/access_controls", label: "ACL", app: "access_controls" },
    { to: "/call_block", label: "Block", app: "call_block" },
    { to: "/call_centers", label: "Call Centers", app: "call_centers" },
    { to: "/fifo", label: "Fifo", app: "fifo" },
    { to: "/bridges", label: "Bridges", app: "bridges" },
    { to: "/streams", label: "Streams", app: "streams" },
    { to: "/phrases", label: "Phrases", app: "phrases" },
    { to: "/pin_numbers", label: "PINs", app: "pin_numbers" },
    { to: "/destinations", label: "Destinations", app: "destinations" },
    { to: "/follow_me", label: "FollowMe", app: "follow_me" },
    { to: "/music_on_hold", label: "MOH", app: "music_on_hold" },
    { to: "/call_flows", label: "Call Flows", app: "call_flows" },
    { to: "/fax", label: "Fax", app: "fax" },
    { to: "/fax_queue", label: "Fax Queue", app: "fax_queue" },
    { to: "/vars", label: "Vars", app: "vars" },
    { to: "/number_translations", label: "Number Trans", app: "number_translations" },
    { to: "/call_broadcasts", label: "Broadcasts", app: "call_broadcast" },
    { to: "/emergency_logs", label: "Emergency", app: "emergency_logs" },
    { to: "/voicemail_greetings", label: "VM Greetings", app: "voicemail_greetings" },
    { to: "/extension_settings", label: "Ext Settings", app: "extension_settings" },
    { to: "/email_queue", label: "Email Queue", app: "email_queue" },
    { to: "/sofia_global_settings", label: "Sofia Global", app: "sofia_global_settings" },
    { to: "/feature_codes", label: "Feature Codes", app: "feature_codes" },
    { to: "/database_transactions", label: "DB Transactions", app: "database_transactions" },
    { to: "/event_guard", label: "Event Guard", app: "event_guard" },
    { to: "/modules", label: "Modules", app: "modules" },
    { to: "/active_calls", label: "Active Calls", app: "active_calls" },
    { to: "/registrations", label: "Registrations", app: "registrations" },
    { to: "/operator_panel", label: "Operator", app: "operator_panel" },
    { to: "/logs", label: "Logs", app: "log_viewer" },
    { to: "/domain_limits", label: "Limits", app: "domain_limits" },
    { to: "/users", label: "Users & Groups", app: "users" },
    { to: "/domains", label: "Tenants", app: "domains" },
    { to: "/plugins", label: "Plugins", app: "plugins" },
  ];
  const mobileAdvanced: { to: string; label: string; app: string }[] = [
    { to: "/ivr", label: "IVR Menus", app: "ivr_menus" },
    { to: "/sip_profiles", label: "SIP Profiles", app: "sip_profiles" },
    { to: "/ring_groups", label: "Ring Groups", app: "ring_groups" },
    { to: "/time_conditions", label: "Time Conditions", app: "time_conditions" },
    { to: "/voicemails", label: "Voicemails", app: "voicemails" },
    { to: "/conferences", label: "Conferences", app: "conferences" },
    { to: "/call_flows", label: "Call Flows", app: "call_flows" },
    { to: "/music_on_hold", label: "MOH", app: "music_on_hold" },
    { to: "/fifo", label: "Fifo", app: "fifo" },
    { to: "/call_centers", label: "Call Centers", app: "call_centers" },
    { to: "/access_controls", label: "ACL", app: "access_controls" },
    { to: "/active_calls", label: "Active Calls", app: "active_calls" },
    { to: "/operator_panel", label: "Operator", app: "operator_panel" },
    { to: "/logs", label: "Logs", app: "log_viewer" },
    { to: "/users", label: "Users & Groups", app: "users" },
    { to: "/domains", label: "Tenants", app: "domains" },
    { to: "/plugins", label: "Plugins", app: "plugins" },
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
          <WebPhoneButton onOpen={()=>setWebPhoneOpen(true)} />
          <WebPhonePopup open={webPhoneOpen} onOpenChange={setWebPhoneOpen} />
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
                  {desktopAdvanced.filter((it) => isEnabled(it.app)).map((it) => (
                    <Link
                      key={it.to}
                      to={it.to}
                      className={`rounded px-3 py-2 text-sm ${loc.pathname === it.to ? "bg-muted font-medium" : "hover:bg-muted"}`}
                    >
                      {it.label}
                    </Link>
                  ))}
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
                {mobileAdvanced.filter((it) => isEnabled(it.app)).map((it) => (
                  <Link
                    key={it.to}
                    to={it.to}
                    onClick={() => setOpen(false)}
                    className="rounded px-3 py-3 text-base hover:bg-muted border"
                  >
                    {it.label}
                  </Link>
                ))}
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
            <Route path="/apps" element={<Apps />} />
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
      <BrowserRouter>
        <WebPhoneProvider>
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
        </WebPhoneProvider>
      </BrowserRouter>
    </ThemeProvider>
  );
}
