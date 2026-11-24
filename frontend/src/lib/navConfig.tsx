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
  PhoneCall, UsersRound, List, PhoneOutgoing,
  BookUser, Disc3, CircleDot, CreditCard, CircleDot as DefaultNavIcon,
} from "lucide-react";

export type NavItem = { to: string; label: string; icon: ComponentType<{ className?: string }> };
export type NavGroup = { label: string; items: NavItem[] };

const ICONS: Record<string, ComponentType<{ className?: string }>> = {
  LayoutDashboard, Phone, Split, Network, Smartphone, BarChart3,
  MapPin, ListTree, PhoneForwarded, Clock, GitBranch, PhoneIncoming,
  Hash, Languages, KeyRound, Ban, MessageSquare,
  Presentation, Layers, SlidersHorizontal, Building, Headphones,
  ListOrdered, Printer, Inbox, Voicemail, Mic, Radio, Music, Megaphone,
  Cable, Globe, ArrowLeftRight, Shield,
  Activity, UserCheck, MonitorSpeaker, ScrollText, Siren, BellRing,
  Building2, Users, Puzzle, Boxes, Variable, Settings, Database, Mail, Gauge, Server,
  PhoneCall, UsersRound, List, PhoneOutgoing,
  BookUser, Disc3, CircleDot, CreditCard,
};

export function iconFor(name: string | null | undefined): ComponentType<{ className?: string }> {
  return (name && ICONS[name]) || DefaultNavIcon;
}

export interface MenuRow {
  menu_item_uuid: string;
  parent_uuid?: string | null;
  title: string;
  path?: string | null;
  icon?: string | null;
  item_order?: number;
  groups?: string | null;
  enabled?: boolean;
}

/** Build nav groups from menu_items rows. Returns [] when empty so callers fall back to static nav. */
export function buildDynamicNav(rows: MenuRow[]): NavGroup[] {
  const items = (rows ?? []).filter((r) => r.enabled !== false);
  if (items.length === 0) return [];
  const byId = new Map(items.map((r) => [r.menu_item_uuid, r]));
  const tops = items
    .filter((r) => !r.parent_uuid || !byId.has(r.parent_uuid))
    .sort((a, b) => (a.item_order ?? 0) - (b.item_order ?? 0));
  const groups: NavGroup[] = [];
  for (const t of tops) {
    const kids = items
      .filter((r) => r.parent_uuid === t.menu_item_uuid && r.path)
      .sort((a, b) => (a.item_order ?? 0) - (b.item_order ?? 0));
    if (t.path) {
      groups.push({ label: t.title, items: [{ to: t.path, label: t.title, icon: iconFor(t.icon) }] });
    } else if (kids.length > 0) {
      groups.push({
        label: t.title,
        items: kids.map((k) => ({ to: k.path as string, label: k.title, icon: iconFor(k.icon) })),
      });
    }
  }
  return groups;
}

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
      { to: "/call_forward", label: "Call Forward", icon: PhoneForwarded },
      { to: "/contacts", label: "Contacts", icon: BookUser },
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
      { to: "/recordings", label: "Recordings", icon: Disc3 },
      { to: "/call_recordings", label: "Call Recordings", icon: CircleDot },
      { to: "/call_broadcasts", label: "Broadcasts", icon: Megaphone },
    ],
  },
  {
    label: "Network",
    items: [
      { to: "/sip_profiles", label: "SIP Profiles", icon: Cable },
      { to: "/device_profiles", label: "Device Profiles", icon: CreditCard },
      { to: "/sofia_global_settings", label: "Sofia Global", icon: Globe },
      { to: "/bridges", label: "Bridges", icon: ArrowLeftRight },
      { to: "/access_controls", label: "Access Controls", icon: Shield },
    ],
  },
  {
    label: "Status",
    items: [
      { to: "/active_calls", label: "Active Calls", icon: Activity },
      { to: "/active_conferences", label: "Active Conferences", icon: UsersRound },
      { to: "/call_center_active", label: "Call Center Active", icon: PhoneCall },
      { to: "/fifo_list", label: "FIFO List", icon: List },
      { to: "/click_to_call", label: "Click to Call", icon: PhoneOutgoing },
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
      { to: "/groups", label: "Groups", icon: Shield },
      { to: "/plugins", label: "Plugins", icon: Puzzle },
      { to: "/modules", label: "Modules", icon: Boxes },
      { to: "/vars", label: "Vars", icon: Variable },
      { to: "/extension_settings", label: "Extension Settings", icon: Settings },
      { to: "/database_transactions", label: "DB Transactions", icon: Database },
      { to: "/email_queue", label: "Email Queue", icon: Mail },
      { to: "/domain_limits", label: "Domain Limits", icon: Gauge },
      { to: "/webrtc_settings", label: "WebRTC", icon: Smartphone },
      { to: "/default_settings", label: "Default Settings", icon: Settings },
      { to: "/domain_settings", label: "Domain Settings", icon: Settings },
      { to: "/email_templates", label: "Email Templates", icon: Mail },
      { to: "/menu_items", label: "Menu Items", icon: List },
      { to: "/menus", label: "Menus", icon: List },
      { to: "/user_logs", label: "User Logs", icon: ScrollText },
      { to: "/notifications", label: "Notifications", icon: BellRing },
      { to: "/software", label: "Software", icon: Boxes },
      { to: "/databases", label: "Databases", icon: Database },
      { to: "/services", label: "Services", icon: Server },
      { to: "/user_settings", label: "My Settings", icon: UserCheck },
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
