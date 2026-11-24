import { FullCrudPage } from "../components/FullCrudPage";
import { CRUD_CONFIGS } from "../lib/fieldConfigs";

export function PinNumbers() {
  return <FullCrudPage config={CRUD_CONFIGS.pin_numbers} />;
}
