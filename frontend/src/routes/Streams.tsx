import { FullCrudPage } from "../components/FullCrudPage";
import { CRUD_CONFIGS } from "../lib/fieldConfigs";

export function Streams() {
  return <FullCrudPage config={CRUD_CONFIGS.streams} />;
}
