import { FullCrudPage } from "../components/FullCrudPage";
import { CRUD_CONFIGS } from "../lib/fieldConfigs";

export function Notifications() {
  return <FullCrudPage config={CRUD_CONFIGS.notifications} />;
}
