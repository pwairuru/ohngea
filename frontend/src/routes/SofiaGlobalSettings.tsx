import { FullCrudPage } from "../components/FullCrudPage";
import { CRUD_CONFIGS } from "../lib/fieldConfigs";

export function SofiaGlobalSettings() {
  return <FullCrudPage config={CRUD_CONFIGS.sofia_global_settings} />;
}
