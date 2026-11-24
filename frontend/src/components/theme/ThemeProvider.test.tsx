import "@testing-library/jest-dom";
import * as React from "react";
import { describe, it, expect, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { ThemeProvider, ThemeToggle, useTheme } from "./ThemeProvider";

function Probe({ value }: { value: string }) {
  const { setTheme } = useTheme();
  return <button data-testid="set-theme" onClick={() => setTheme(value as never)}>{value}</button>;
}

describe("ThemeProvider fusion compact", () => {
  beforeEach(() => {
    localStorage.clear();
    document.documentElement.className = "";
    if (!window.matchMedia) {
      Object.defineProperty(window, "matchMedia", {
        writable: true,
        value: () => ({ matches: false, addEventListener: () => {}, removeEventListener: () => {} }),
      });
    }
  });

  it("offers fusion + fusion-dark in selector", () => {
    render(
      <ThemeProvider>
        <ThemeToggle />
      </ThemeProvider>
    );
    const select = screen.getByTestId("theme-select") as HTMLSelectElement;
    const values = Array.from(select.options).map((o) => o.value);
    expect(values).toContain("fusion");
    expect(values).toContain("fusion-dark");
  });

  it("applies theme-fusion class to documentElement", () => {
    render(
      <ThemeProvider>
        <Probe value="fusion" />
      </ThemeProvider>
    );
    fireEvent.click(screen.getAllByTestId("set-theme")[0]);
    expect(document.documentElement.classList.contains("theme-fusion")).toBe(true);
  });

  it("applies theme-fusion-dark + dark classes", () => {
    render(
      <ThemeProvider>
        <Probe value="fusion-dark" />
      </ThemeProvider>
    );
    const btns = screen.getAllByTestId("set-theme");
    fireEvent.click(btns[btns.length - 1]);
    expect(document.documentElement.classList.contains("theme-fusion-dark")).toBe(true);
    expect(document.documentElement.classList.contains("dark")).toBe(true);
  });
});
