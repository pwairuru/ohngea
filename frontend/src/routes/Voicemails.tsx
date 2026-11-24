import { FullCrudPage } from "../components/FullCrudPage";
import { CRUD_CONFIGS } from "../lib/fieldConfigs";

export function Voicemails() {
  return <FullCrudPage config={CRUD_CONFIGS.voicemails} />;
}
