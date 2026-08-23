import "@testing-library/jest-dom";
import * as React from "react";
import { describe, it, expect, beforeEach } from "vitest";
import { render, screen, fireEvent, act } from "@testing-library/react";
import { toast, toastError, clearToasts, Toaster } from "./toast";

describe("Toaster", () => {
  beforeEach(() => clearToasts());
  it("renders a toast on toast()", async () => {
    render(<Toaster />);
    expect(screen.queryByTestId("toaster")).toBeNull();
    await act(async () => { toast("Extension 1001 created", { variant: "success" }); });
    expect(screen.getByTestId("toast")).toHaveTextContent("Extension 1001 created");
  });
  it("renders error variant message via toastError", async () => {
    render(<Toaster />);
    await act(async () => { toastError(new Error("409 duplicate")); });
    expect(screen.getByTestId("toast")).toHaveTextContent("409 duplicate");
  });
  it("dismisses on close click", async () => {
    render(<Toaster />);
    await act(async () => { toast("hi"); });
    await act(async () => { fireEvent.click(screen.getByLabelText("Dismiss")); });
    expect(screen.queryByText("hi")).toBeNull();
  });
});
