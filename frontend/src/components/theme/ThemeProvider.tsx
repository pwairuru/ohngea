import * as React from "react";

type Theme = "light" | "dark" | "system" | "ohngea" | "ohngea-dark" | "fusion" | "fusion-dark";

const ThemeContext = React.createContext<{ theme: Theme; setTheme: (t: Theme) => void }>({ theme: "system", setTheme: () => {} });

export function ThemeProvider({ children }: { children: React.ReactNode }) {
  const [theme, setTheme] = React.useState<Theme>(() => (localStorage.getItem("theme") as Theme) || "system");

  React.useEffect(() => {
    const root = document.documentElement;
    root.classList.remove("dark", "theme-ohngea", "theme-ohngea-dark", "theme-fusion", "theme-fusion-dark");
    if (theme === "ohngea") {
      root.classList.add("theme-ohngea");
    } else if (theme === "ohngea-dark") {
      // ohngea-dark is itself dark — also add .dark so dark: utilities work
      root.classList.add("theme-ohngea-dark", "dark");
    } else if (theme === "fusion") {
      root.classList.add("theme-fusion");
    } else if (theme === "fusion-dark") {
      root.classList.add("theme-fusion-dark", "dark");
    } else {
      const systemDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
      const isDark = theme === "dark" || (theme === "system" && systemDark);
      if (isDark) root.classList.add("dark");
    }
    localStorage.setItem("theme", theme);
  }, [theme]);

  return <ThemeContext.Provider value={{ theme, setTheme }}>{children}</ThemeContext.Provider>;
}

export function useTheme() {
  return React.useContext(ThemeContext);
}

export function ThemeToggle() {
  const { theme, setTheme } = useTheme();
  const order: Theme[] = ["light", "dark", "ohngea", "ohngea-dark", "fusion", "fusion-dark", "system"];
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
        {theme === "dark" ? "☀️" : theme === "ohngea" ? "🟦" : theme === "ohngea-dark" ? "🌊" : theme === "fusion" ? "📞" : theme === "fusion-dark" ? "🌃" : theme === "system" ? "💻" : "🌙"}
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
        <option value="fusion">Fusion</option>
        <option value="fusion-dark">Fusion Dark</option>
        <option value="system">System</option>
      </select>
    </div>
  );
}
