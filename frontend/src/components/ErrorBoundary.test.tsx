import { describe, it, expect } from "vitest";
import "@testing-library/jest-dom";
import { render, screen, fireEvent } from "@testing-library/react";
import { MemoryRouter, Routes, Route, Link } from "react-router-dom";
import { ErrorBoundary, RouteErrorBoundary } from "./ErrorBoundary";

function Boom(): React.ReactElement {
  throw new Error("boom");
}

describe("ErrorBoundary route reset", () => {
  it("resets when resetKey changes (navigation)", () => {
    const { rerender } = render(
      <ErrorBoundary resetKey="/a"><Boom /></ErrorBoundary>
    );
    expect(screen.getByTestId("error-boundary")).toBeInTheDocument();
    // navigate: same boundary with new key clears the error
    rerender(
      <ErrorBoundary resetKey="/b"><div>recovered</div></ErrorBoundary>
    );
    expect(screen.getByText("recovered")).toBeInTheDocument();
  });

  it("RouteErrorBoundary recovers on route change", async () => {
    render(
      <MemoryRouter initialEntries={["/bad"]}>
        <RouteErrorBoundary>
          <Routes>
            <Route path="/bad" element={<Boom />} />
            <Route path="/good" element={<div>good page <Link to="/good">g</Link></div>} />
          </Routes>
        </RouteErrorBoundary>
        <Link to="/good">go good</Link>
      </MemoryRouter>
    );
    expect(screen.getByTestId("error-boundary")).toBeInTheDocument();
    fireEvent.click(screen.getByText("go good"));
    expect(await screen.findByText(/good page/)).toBeInTheDocument();
  });
});
