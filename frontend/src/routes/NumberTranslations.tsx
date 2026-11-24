import { FullCrudPage } from "../components/FullCrudPage";
import { CRUD_CONFIGS } from "../lib/fieldConfigs";

export function NumberTranslations() {
  return <FullCrudPage config={CRUD_CONFIGS.number_translations} />;
}
