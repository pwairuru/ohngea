import * as React from "react";

type Theme = "light" | "dark" | "system";

const ThemeContext = React.createContext<{ theme: Theme; setTheme: (t: Theme) => void }>({ theme: "system", setTheme: () => {} });

export function ThemeProvider({ children }: { children: React.ReactNode }) {
  const [theme, setTheme] = React.useState<Theme>(() => (localStorage.getItem("theme") as Theme) || "system");

  React.useEffect(() => {
    const root = document.documentElement;
    const systemDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
    const isDark = theme === "dark" || (theme === "system" && systemDark);
    root.classList.toggle("dark", isDark);
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
        const style = document.getElementById("tenant-theme") || document.createElement("style");
        style.id = "tenant-theme";
        style.textContent = `:root{--primary:${data.primary_color};--background:${data.background};--radius:${data.radius}}`;
        document.head.appendChild(style);
      })
      .catch(() => {});
  }, []);

  return <ThemeContext.Provider value={{ theme, setTheme }}>{children}</ThemeContext.Provider>;
}

export function useTheme() {
  return React.useContext(ThemeContext);
}

export function ThemeToggle() {
  const { theme, setTheme } = useTheme();
  return (
    <button
      data-testid="theme-toggle"
      onClick={() => setTheme(theme === "dark" ? "light" : "dark")}
      className="inline-flex h-10 w-10 items-center justify-center rounded-md border min-h-[44px] min-w-[44px]"
      aria-label="Toggle theme"
    >
      {theme === "dark" ? "☀️" : "🌙"}
    </button>
  );
}
