import { FullCrudPage } from "../components/FullCrudPage";
import { CRUD_CONFIGS } from "../lib/fieldConfigs";

export function IvrMenus() {
  return <FullCrudPage config={CRUD_CONFIGS.ivr_menus} />;
}
