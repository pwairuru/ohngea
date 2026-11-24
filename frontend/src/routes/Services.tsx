import { FullCrudPage } from "../components/FullCrudPage";
import { CRUD_CONFIGS } from "../lib/fieldConfigs";

export function Services() {
  return <FullCrudPage config={CRUD_CONFIGS.services} />;
}
