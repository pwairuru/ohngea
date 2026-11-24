import { FullCrudPage } from "../components/FullCrudPage";
import { CRUD_CONFIGS } from "../lib/fieldConfigs";

export function ConferenceCenters() {
  return <FullCrudPage config={CRUD_CONFIGS.conference_centers} />;
}
