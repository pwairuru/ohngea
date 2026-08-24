import * as React from "react";

type Theme = "light" | "dark" | "system" | "ohngea" | "ohngea-dark";

const ThemeContext = React.createContext<{ theme: Theme; setTheme: (t: Theme) => void }>({ theme: "system", setTheme: () => {} });

export function ThemeProvider({ children }: { children: React.ReactNode }) {
  const [theme, setTheme] = React.useState<Theme>(() => (localStorage.getItem("theme") as Theme) || "system");

  React.useEffect(() => {
    const root = document.documentElement;
    root.classList.remove("dark", "theme-ohngea", "theme-ohngea-dark");
    if (theme === "ohngea") {
      root.classList.add("theme-ohngea");
    } else if (theme === "ohngea-dark") {
      root.classList.add("theme-ohngea-dark");
    } else {
      const systemDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
      const isDark = theme === "dark" || (theme === "system" && systemDark);
      if (isDark) root.classList.add("dark");
    }
    localStorage.setItem("theme", theme);
  }, [theme]);

  // per-tenant override fetch if domain_id stored
  React.useEffect(() => {
    const domainId = localStorage.getItem("domain_id");
    if (!domainId) return;
    fetch(`/api/v1/domains/${domainId}/theme`)
      .then((r) => (r.ok ? r.json() : null))
      .then((data) => {
        if (!data) return;
        // validate primary_color before injection (hex #rrggbb, hsl(), or bare triple "174 72% 40%")
        const isHex = typeof data.primary_color === "string" && /^#[0-9a-fA-F]{6}$/.test(data.primary_color);
        const isHsl = typeof data.primary_color === "string" && /^hsl\(/i.test(data.primary_color);
        const isHslTriple = typeof data.primary_color === "string" && /^\d+(\.\d+)?\s+\d+(\.\d+)?%\s+\d+(\.\d+)?%$/.test(data.primary_color.trim());
        if (data.primary_color && !isHex && !isHsl && !isHslTriple) {
          console.warn("Theme fetch: invalid primary_color", data.primary_color);
          return;
        }
        const style = document.getElementById("tenant-theme") || document.createElement("style");
        style.id = "tenant-theme";
        style.textContent = `:root{--primary:${data.primary_color};--background:${data.background};--radius:${data.radius}}`;
        document.head.appendChild(style);
      })
      .catch((e) => console.warn("Theme fetch failed", e));
  }, []);

  return <ThemeContext.Provider value={{ theme, setTheme }}>{children}</ThemeContext.Provider>;
}

export function useTheme() {
  return React.useContext(ThemeContext);
}

export function ThemeToggle() {
  const { theme, setTheme } = useTheme();
  const order: Theme[] = ["light", "dark", "ohngea", "ohngea-dark", "system"];
  const cycle = () => {
    const idx = order.indexOf(theme);
    const next = order[(idx + 1) % order.length];
    setTheme(next);
  };
  return (
    <div className="flex items-center gap-1">
      <button
        data-testid="theme-toggle"
        onClick={cycle}
        className="inline-flex h-10 w-10 items-center justify-center rounded-md border min-h-[44px] min-w-[44px]"
        aria-label="Toggle theme"
      >
        {theme === "dark" ? "☀️" : theme === "ohngea" ? "🟦" : theme === "ohngea-dark" ? "🌊" : theme === "system" ? "💻" : "🌙"}
      </button>
      <select
        aria-label="Theme"
        data-testid="theme-select"
        value={theme}
        onChange={(e) => setTheme(e.target.value as Theme)}
        className="h-10 rounded-md border bg-background px-2 text-sm"
      >
        <option value="light">Light</option>
        <option value="dark">Dark</option>
        <option value="ohngea">Ohngea</option>
        <option value="ohngea-dark">Ohngea Dark</option>
        <option value="system">System</option>
      </select>
    </div>
  );
}
