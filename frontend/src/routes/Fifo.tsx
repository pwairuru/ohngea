import { FullCrudPage } from "../components/FullCrudPage";
import { CRUD_CONFIGS } from "../lib/fieldConfigs";

export function Fifo() {
  return <FullCrudPage config={CRUD_CONFIGS.fifo} />;
}
