import { FullCrudPage } from "../components/FullCrudPage";
import { CRUD_CONFIGS } from "../lib/fieldConfigs";

export function CallBroadcasts() {
  return <FullCrudPage config={CRUD_CONFIGS.call_broadcast} />;
}
