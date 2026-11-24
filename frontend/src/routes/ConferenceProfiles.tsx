import { FullCrudPage } from "../components/FullCrudPage";
import { CRUD_CONFIGS } from "../lib/fieldConfigs";

export function ConferenceProfiles() {
  return <FullCrudPage config={CRUD_CONFIGS.conference_profiles} />;
}
