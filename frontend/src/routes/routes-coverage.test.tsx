import { describe, it, expect, vi } from "vitest";
import "@testing-library/jest-dom";
import { render, screen, fireEvent } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import { CRUD_CONFIGS } from "../lib/fieldConfigs";

vi.mock("../lib/api", () => ({
  apiFetch: vi.fn(() => Promise.resolve({ data: [], total: 0 })),
}));

const pages: Array<{ path: string; loader: () => Promise<any>; heading: RegExp }> = [
  { path: "AccessControls", loader: () => import("./AccessControls"), heading: /Access Controls/i },
  { path: "Bridges", loader: () => import("./Bridges"), heading: /Bridges/i },
  { path: "CallBlock", loader: () => import("./CallBlock"), heading: /Call Block/i },
  { path: "CallBroadcasts", loader: () => import("./CallBroadcasts"), heading: /Broadcast/i },
  { path: "CallCenters", loader: () => import("./CallCenters"), heading: /Call Centers/i },
  { path: "CallFlows", loader: () => import("./CallFlows"), heading: /Call Flows/i },
  { path: "CallRecordings", loader: () => import("./CallRecordings"), heading: /Call Recordings/i },
  { path: "ConferenceCenters", loader: () => import("./ConferenceCenters"), heading: /Conference Centers/i },
  { path: "ConferenceControls", loader: () => import("./ConferenceControls"), heading: /Conference Controls/i },
  { path: "ConferenceProfiles", loader: () => import("./ConferenceProfiles"), heading: /Conference Profiles/i },
  { path: "Conferences", loader: () => import("./Conferences"), heading: /Conferences/i },
  { path: "Contacts", loader: () => import("./Contacts"), heading: /Contacts/i },
  { path: "DatabaseTransactions", loader: () => import("./DatabaseTransactions"), heading: /Database Transactions/i },
  { path: "DefaultSettings", loader: () => import("./DefaultSettings"), heading: /Default Settings/i },
  { path: "Destinations", loader: () => import("./Destinations"), heading: /Destinations/i },
  { path: "DomainSettings", loader: () => import("./DomainSettings"), heading: /Domain Settings/i },
  { path: "Destinations", loader: () => import("./Destinations"), heading: /Destinations/i },
  { path: "DeviceProfiles", loader: () => import("./DeviceProfiles"), heading: /Device Profiles/i },
  { path: "DomainLimits", loader: () => import("./DomainLimits"), heading: /Domain Limits/i },
  { path: "EmailQueue", loader: () => import("./EmailQueue"), heading: /Email Queue/i },
  { path: "EmailTemplates", loader: () => import("./EmailTemplates"), heading: /Email Templates/i },
  { path: "EmergencyLogs", loader: () => import("./EmergencyLogs"), heading: /Emergency/i },
  { path: "EventGuard", loader: () => import("./EventGuard"), heading: /Event Guard/i },
  { path: "ExtensionSettings", loader: () => import("./ExtensionSettings"), heading: /Extension Settings/i },
  { path: "Fax", loader: () => import("./Fax"), heading: /Fax/i },
  { path: "FaxQueue", loader: () => import("./FaxQueue"), heading: /Fax Queue/i },
  { path: "FeatureCodes", loader: () => import("./FeatureCodes"), heading: /Feature Codes/i },
  { path: "Fifo", loader: () => import("./Fifo"), heading: /Fifo/i },
  { path: "FollowMe", loader: () => import("./FollowMe"), heading: /Follow Me/i },
  { path: "Groups", loader: () => import("./Groups"), heading: /Groups/i },
  { path: "IvrMenus", loader: () => import("./IvrMenus"), heading: /IVR Menus/i },
  { path: "MenuItems", loader: () => import("./MenuItems"), heading: /Menu Items/i },
  { path: "Menus", loader: () => import("./Menus"), heading: /Menus/i },
  { path: "Modules", loader: () => import("./Modules"), heading: /Modules/i },
  { path: "MusicOnHold", loader: () => import("./MusicOnHold"), heading: /Music on Hold/i },
  { path: "Notifications", loader: () => import("./Notifications"), heading: /Notifications/i },
  { path: "NumberTranslations", loader: () => import("./NumberTranslations"), heading: /Number Translations/i },
  { path: "Phrases", loader: () => import("./Phrases"), heading: /Phrases/i },
  { path: "PinNumbers", loader: () => import("./PinNumbers"), heading: /PIN/i },
  { path: "Recordings", loader: () => import("./Recordings"), heading: /Recordings/i },
  { path: "RingGroups", loader: () => import("./RingGroups"), heading: /Ring Groups/i },
  { path: "Services", loader: () => import("./Services"), heading: /Services/i },
  { path: "SofiaGlobalSettings", loader: () => import("./SofiaGlobalSettings"), heading: /Sofia/i },
  { path: "Streams", loader: () => import("./Streams"), heading: /Streams/i },
  { path: "TimeConditions", loader: () => import("./TimeConditions"), heading: /Time Conditions/i },
  { path: "UserLogs", loader: () => import("./UserLogs"), heading: /User Logs/i },
  { path: "Vars", loader: () => import("./Vars"), heading: /Vars/i },
  { path: "VoicemailGreetings", loader: () => import("./VoicemailGreetings"), heading: /Greetings/i },
  { path: "Voicemails", loader: () => import("./Voicemails"), heading: /Voicemails/i },
];

describe("route pages full parity", () => {
  for (const p of pages) {
    it(`${p.path} renders heading, search and Show Advanced`, async () => {
      const mod = await p.loader();
      const Comp = mod[p.path];
      expect(Comp, `${p.path} export`).toBeDefined();
      const { unmount } = render(
        <MemoryRouter>
          <Comp />
        </MemoryRouter>
      );
      expect(screen.getAllByText(p.heading).length, `${p.path} heading`).toBeGreaterThanOrEqual(1);
      expect(
        screen.queryByPlaceholderText(/Search/i),
        `${p.path} search`
      ).toBeInTheDocument();
      // open create dialog and check advanced toggle exists
      const addBtn = screen.queryByText(/Add /);
      const cfg = Object.values(CRUD_CONFIGS).find((c) => p.heading.test(c.title));
      if (cfg?.readOnly) {
        expect(addBtn, `${p.path} read-only has no add`).toBeNull();
        unmount();
        return;
      }
      expect(addBtn, `${p.path} add button`).toBeInTheDocument();
      if (addBtn) {
        fireEvent.click(addBtn as HTMLElement);
        if (cfg && cfg.advancedFields.length > 0) {
          expect(
            screen.queryByText(/Show Advanced/),
            `${p.path} advanced toggle`
          ).toBeInTheDocument();
        } else {
          // no advanced fields: key fields must render instead
          const firstLabel = new RegExp(cfg?.keyFields[0]?.label ?? "Name");
          expect(screen.queryAllByText(firstLabel).length, `${p.path} key field`).toBeGreaterThanOrEqual(1);
        }
      }
      unmount();
    });
  }
});
