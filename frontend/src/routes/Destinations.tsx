import { FullCrudPage } from "../components/FullCrudPage";
import { CRUD_CONFIGS } from "../lib/fieldConfigs";

export function Destinations() {
  return <FullCrudPage config={CRUD_CONFIGS.destinations} />;
}
