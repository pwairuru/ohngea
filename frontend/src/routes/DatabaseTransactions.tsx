import { FullCrudPage } from "../components/FullCrudPage";
import { CRUD_CONFIGS } from "../lib/fieldConfigs";

export function DatabaseTransactions() {
  return <FullCrudPage config={CRUD_CONFIGS.database_transactions} />;
}
