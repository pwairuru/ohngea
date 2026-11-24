import { FullCrudPage } from "../components/FullCrudPage";
import { CRUD_CONFIGS } from "../lib/fieldConfigs";

export function Conferences() {
  return <FullCrudPage config={CRUD_CONFIGS.conferences} />;
}
