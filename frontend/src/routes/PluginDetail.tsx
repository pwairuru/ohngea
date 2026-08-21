import * as React from "react";
import { useParams } from "react-router-dom";

export function PluginDetail() {
  const { name } = useParams();
  const [Comp, setComp] = React.useState<any>(null);
  React.useEffect(() => {
    if (!name) return;
    import(`../plugins/${name}/App.tsx`)
      .then((m) => setComp(() => m.default))
      .catch(() => setComp(() => () => <div className="p-4">Plugin {name} UI not found — check frontend/src/plugins/{name}/App.tsx</div>));
  }, [name]);
  if (!Comp) return <div className="p-4">Loading plugin {name}...</div>;
  return <Comp />;
}
