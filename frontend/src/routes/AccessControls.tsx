import { FullCrudPage } from "../components/FullCrudPage";
import { CRUD_CONFIGS } from "../lib/fieldConfigs";

export function AccessControls() {
  return <FullCrudPage config={CRUD_CONFIGS.access_controls} />;
}
