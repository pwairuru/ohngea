import { FullCrudPage } from "../components/FullCrudPage";
import { CRUD_CONFIGS } from "../lib/fieldConfigs";

export function EventGuard() {
  return <FullCrudPage config={CRUD_CONFIGS.event_guard} />;
}
