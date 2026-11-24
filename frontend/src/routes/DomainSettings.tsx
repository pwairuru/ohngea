import { FullCrudPage } from "../components/FullCrudPage";
import { CRUD_CONFIGS } from "../lib/fieldConfigs";

export function DomainSettings() {
  return <FullCrudPage config={CRUD_CONFIGS.domain_settings} />;
}
