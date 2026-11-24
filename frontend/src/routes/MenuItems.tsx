import { FullCrudPage } from "../components/FullCrudPage";
import { CRUD_CONFIGS } from "../lib/fieldConfigs";

export function MenuItems() {
  return <FullCrudPage config={CRUD_CONFIGS.menu_items} />;
}
