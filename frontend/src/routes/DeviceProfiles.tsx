import { FullCrudPage } from "../components/FullCrudPage";
import { CRUD_CONFIGS } from "../lib/fieldConfigs";

export function DeviceProfiles() {
  return <FullCrudPage config={CRUD_CONFIGS.device_profiles} />;
}
