import { FullCrudPage } from "../components/FullCrudPage";
import { CRUD_CONFIGS } from "../lib/fieldConfigs";

export function MusicOnHold() {
  return <FullCrudPage config={CRUD_CONFIGS.music_on_hold} />;
}
