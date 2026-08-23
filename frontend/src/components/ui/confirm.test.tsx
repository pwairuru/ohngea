import "@testing-library/jest-dom";
import * as React from "react";
import { describe, it, expect } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { ConfirmProvider, useConfirm } from "./confirm";

function Probe({ onResult }: { onResult: (v: boolean) => void }) {
  const confirm = useConfirm();
  return <button onClick={() => confirm({ title: "Delete extension 1001?", confirmLabel: "Delete" }).then(onResult)}>run</button>;
}

describe("ConfirmDialog", () => {
  it("resolves true on confirm", async () => {
    let result: boolean | null = null;
    render(<ConfirmProvider><Probe onResult={(v) => (result = v)} /></ConfirmProvider>);
    fireEvent.click(screen.getByText("run"));
    expect(await screen.findByTestId("confirm-dialog")).toHaveTextContent("Delete extension 1001?");
    fireEvent.click(screen.getByTestId("confirm-ok"));
    await waitFor(() => expect(result).toBe(true));
  });
  it("resolves false on cancel", async () => {
    let result: boolean | null = null;
    render(<ConfirmProvider><Probe onResult={(v) => (result = v)} /></ConfirmProvider>);
    fireEvent.click(screen.getByText("run"));
    fireEvent.click(await screen.findByTestId("confirm-cancel"));
    await waitFor(() => expect(result).toBe(false));
  });
});
