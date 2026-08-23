import "@testing-library/jest-dom/vitest";
import { render, screen } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
vi.stubGlobal("navigator", { mediaDevices: { enumerateDevices: async()=>[{deviceId:"a", kind:"audioinput", label:"Mic"} as any], getUserMedia: async()=>({}) as any }});
import { DevicePicker } from "./DevicePicker";
describe("DevicePicker", ()=>{ it("renders", async()=>{ render(<DevicePicker />); expect(await screen.findByTestId("device-picker")).toBeInTheDocument(); }); });
