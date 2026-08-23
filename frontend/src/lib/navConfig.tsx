import type { ComponentType } from "react";
import {
  LayoutDashboard, Phone, Split, Network, Smartphone, BarChart3,
  MapPin, ListTree, PhoneForwarded, Clock, GitBranch, PhoneIncoming,
  Hash, Languages, KeyRound, Ban, MessageSquare,
  Presentation, Layers, SlidersHorizontal, Building, Headphones,
  ListOrdered, Printer, Inbox, Voicemail, Mic, Radio, Music, Megaphone,
  Cable, Globe, ArrowLeftRight, Shield,
  Activity, UserCheck, MonitorSpeaker, ScrollText, Siren, BellRing,
  Building2, Users, Puzzle, Boxes, Variable, Settings, Database, Mail, Gauge, Server,
} from "lucide-react";

export type NavItem = { to: string; label: string; icon: ComponentType<{ className?: string }> };
export type NavGroup = { label: string; items: NavItem[] };

export const primaryNav: NavItem[] = [
  { to: "/", label: "Dashboard", icon: LayoutDashboard },
  { to: "/extensions", label: "Extensions", icon: Phone },
  { to: "/dialplans", label: "Call Routing", icon: Split },
  { to: "/gateways", label: "Gateways", icon: Network },
  { to: "/devices", label: "Devices", icon: Smartphone },
  { to: "/cdr", label: "Reports", icon: BarChart3 },
];

export const advancedNav: NavGroup[] = [
  {
    label: "Call Routing",
    items: [
      { to: "/destinations", label: "Destinations", icon: MapPin },
      { to: "/ivr", label: "IVR Menus", icon: ListTree },
      { to: "/ring_groups", label: "Ring Groups", icon: PhoneForwarded },
      { to: "/time_conditions", label: "Time Conditions", icon: Clock },
      { to: "/call_flows", label: "Call Flows", icon: GitBranch },
      { to: "/follow_me", label: "Follow Me", icon: PhoneIncoming },
      { to: "/feature_codes", label: "Feature Codes", icon: Hash },
      { to: "/number_translations", label: "Number Translations", icon: Languages },
      { to: "/pin_numbers", label: "PIN Numbers", icon: KeyRound },
      { to: "/call_block", label: "Call Block", icon: Ban },
      { to: "/phrases", label: "Phrases", icon: MessageSquare },
    ],
  },
  {
    label: "Applications",
    items: [
      { to: "/conferences", label: "Conferences", icon: Presentation },
      { to: "/conference_profiles", label: "Conf Profiles", icon: Layers },
      { to: "/conference_controls", label: "Conf Controls", icon: SlidersHorizontal },
      { to: "/conference_centers", label: "Conf Centers", icon: Building },
      { to: "/call_centers", label: "Call Centers", icon: Headphones },
      { to: "/fifo", label: "FIFO", icon: ListOrdered },
      { to: "/fax", label: "Fax", icon: Printer },
      { to: "/fax_queue", label: "Fax Queue", icon: Inbox },
      { to: "/voicemails", label: "Voicemails", icon: Voicemail },
      { to: "/voicemail_greetings", label: "VM Greetings", icon: Mic },
      { to: "/streams", label: "Streams", icon: Radio },
      { to: "/music_on_hold", label: "Music on Hold", icon: Music },
      { to: "/call_broadcasts", label: "Broadcasts", icon: Megaphone },
    ],
  },
  {
    label: "Network",
    items: [
      { to: "/sip_profiles", label: "SIP Profiles", icon: Cable },
      { to: "/sofia_global_settings", label: "Sofia Global", icon: Globe },
      { to: "/bridges", label: "Bridges", icon: ArrowLeftRight },
      { to: "/access_controls", label: "Access Controls", icon: Shield },
    ],
  },
  {
    label: "Status",
    items: [
      { to: "/active_calls", label: "Active Calls", icon: Activity },
      { to: "/registrations", label: "Registrations", icon: UserCheck },
      { to: "/operator_panel", label: "Operator Panel", icon: MonitorSpeaker },
      { to: "/logs", label: "Logs", icon: ScrollText },
      { to: "/emergency_logs", label: "Emergency Logs", icon: Siren },
      { to: "/event_guard", label: "Event Guard", icon: BellRing },
    ],
  },
  {
    label: "System",
    items: [
      { to: "/domains", label: "Tenants", icon: Building2 },
      { to: "/users", label: "Users & Groups", icon: Users },
      { to: "/plugins", label: "Plugins", icon: Puzzle },
      { to: "/modules", label: "Modules", icon: Boxes },
      { to: "/vars", label: "Vars", icon: Variable },
      { to: "/extension_settings", label: "Extension Settings", icon: Settings },
      { to: "/database_transactions", label: "DB Transactions", icon: Database },
      { to: "/email_queue", label: "Email Queue", icon: Mail },
      { to: "/domain_limits", label: "Domain Limits", icon: Gauge },
      { to: "/system", label: "System Status", icon: Server },
    ],
  },
];

export const flatNav: NavItem[] = [...primaryNav, ...advancedNav.flatMap((g) => g.items)];

export const routeMeta: Record<string, { label: string; group?: string }> = Object.fromEntries([
  ...primaryNav.map((i) => [i.to, { label: i.label }] as const),
  ...advancedNav.flatMap((g) => g.items.map((i) => [i.to, { label: i.label, group: g.label }] as const)),
]);

export function findRoute(pathname: string): { label: string; group?: string } | undefined {
  return routeMeta[pathname];
}
