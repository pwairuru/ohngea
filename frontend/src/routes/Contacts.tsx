import { FullCrudPage } from "../components/FullCrudPage";
import { CRUD_CONFIGS } from "../lib/fieldConfigs";

export function Contacts() {
  return <FullCrudPage config={CRUD_CONFIGS.contacts} />;
}
