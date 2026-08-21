export async function loadPlugin(name: string) {
  try {
    // dynamic import plugin UI chunk if enabled
    const mod = await import(`../plugins/${name}/App.tsx`);
    return mod.default;
  } catch {
    return null;
  }
}
