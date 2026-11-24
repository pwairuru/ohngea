import { FullCrudPage } from "../components/FullCrudPage";
import { CRUD_CONFIGS } from "../lib/fieldConfigs";

export function CallBlock() {
  return <FullCrudPage config={CRUD_CONFIGS.call_block} />;
}
