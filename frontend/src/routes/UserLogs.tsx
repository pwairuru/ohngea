import { FullCrudPage } from "../components/FullCrudPage";
import { CRUD_CONFIGS } from "../lib/fieldConfigs";

export function UserLogs() {
  return <FullCrudPage config={CRUD_CONFIGS.user_logs} />;
}
