import { FullCrudPage } from "../components/FullCrudPage";
import { CRUD_CONFIGS } from "../lib/fieldConfigs";

export function Recordings() {
  return <FullCrudPage config={CRUD_CONFIGS.recordings} />;
}
