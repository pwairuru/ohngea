import { FullCrudPage } from "../components/FullCrudPage";
import { CRUD_CONFIGS } from "../lib/fieldConfigs";

export function Software() {
  return <FullCrudPage config={CRUD_CONFIGS.software} />;
}
