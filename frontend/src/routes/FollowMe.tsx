import { FullCrudPage } from "../components/FullCrudPage";
import { CRUD_CONFIGS } from "../lib/fieldConfigs";

export function FollowMe() {
  return <FullCrudPage config={CRUD_CONFIGS.follow_me} />;
}
