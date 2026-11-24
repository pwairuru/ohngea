import { FullCrudPage } from "../components/FullCrudPage";
import { CRUD_CONFIGS } from "../lib/fieldConfigs";

export function FaxQueue() {
  return <FullCrudPage config={CRUD_CONFIGS.fax_queue} />;
}
