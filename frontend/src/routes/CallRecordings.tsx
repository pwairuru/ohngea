import { FullCrudPage } from "../components/FullCrudPage";
import { CRUD_CONFIGS } from "../lib/fieldConfigs";

export function CallRecordings() {
  return <FullCrudPage config={CRUD_CONFIGS.call_recordings} />;
}
