import { FullCrudPage } from "../components/FullCrudPage";
import { CRUD_CONFIGS } from "../lib/fieldConfigs";

export function EmergencyLogs() {
  return <FullCrudPage config={CRUD_CONFIGS.emergency_logs} />;
}
