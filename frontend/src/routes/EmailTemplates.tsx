import { FullCrudPage } from "../components/FullCrudPage";
import { CRUD_CONFIGS } from "../lib/fieldConfigs";

export function EmailTemplates() {
  return <FullCrudPage config={CRUD_CONFIGS.email_templates} />;
}
