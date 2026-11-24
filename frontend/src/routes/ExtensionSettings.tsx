import { FullCrudPage } from "../components/FullCrudPage";
import { CRUD_CONFIGS } from "../lib/fieldConfigs";

export function ExtensionSettings() {
  return <FullCrudPage config={CRUD_CONFIGS.extension_settings} />;
}
