import * as React from "react";
import { BrowserRouter, Routes, Route, Link, useLocation, Navigate } from "react-router-dom";
import { Search } from "lucide-react";
import { ThemeProvider, ThemeToggle } from "./components/theme/ThemeProvider";
import { Toaster } from "./lib/toast";
import { ConfirmProvider } from "./components/ui/confirm";
import { FsStatus } from "./components/FsStatus";
import { Sidebar } from "./components/shell/Sidebar";
import { MobileDrawer } from "./components/shell/MobileDrawer";
import { CommandPalette } from "./components/shell/CommandPalette";
import { UserMenu } from "./components/shell/UserMenu";
import { ErrorBoundary } from "./components/ErrorBoundary";
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
          <button
            onClick={() => window.dispatchEvent(new KeyboardEvent("keydown", { key: "k", ctrlKey: true }))}
            className="hidden md:flex items-center gap-2 rounded-md border px-3 py-1.5 text-sm text-muted-foreground hover:bg-accent hover:text-accent-foreground"
          >
            <Search className="h-4 w-4" />
            <span>Search</span>
            <kbd className="rounded border bg-muted px-1.5 py-0.5 text-xs">Ctrl K</kbd>
          </button>
        </div>
        <div className="flex items-center gap-2">
          <FsStatus />
          <CommandPalette />
          <ThemeToggle />
          <UserMenu />
        </div>
      </header>
      <div className="flex">
        <Sidebar />
        <MobileDrawer open={open} onClose={() => setOpen(false)} />
        <main className="min-w-0 flex-1 bg-muted/20 min-h-[calc(100vh-3.5rem)]">
          <ErrorBoundary>
            <div className="p-4 md:p-6 space-y-4">
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
            </div>
          </ErrorBoundary>
        </main>
      </div>
    </>
  );
}

export default function App() {
  return (
    <ThemeProvider>
      <ConfirmProvider>
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
      </ConfirmProvider>
    </ThemeProvider>
  );
}
