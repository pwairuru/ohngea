import path from "node:path";
import { defineConfig, loadEnv } from "vite";
import react from "@vitejs/plugin-react";
import { VitePWA } from "vite-plugin-pwa";

// API host port follows API_PORT in root .env (single source of truth).
export default defineConfig(({ mode }) => {
  const rootEnv = loadEnv(mode, path.resolve(__dirname, ".."), "");
  const api = `http://localhost:${rootEnv.API_PORT || "8010"}`;
  return {
  plugins: [
    react(),
    VitePWA({
      registerType: "autoUpdate",
      // E2E: inject manifest link under `vite dev` too (default is build-only).
      devOptions: { enabled: true },
      manifest: {
        name: "Ohngea PBX",
        short_name: "Ohngea",
        display: "standalone",
        background_color: "#0F172A",
        theme_color: "#14B8A6",
        description: "Speak. Connect. Ohngea.",
        icons: [
          { src: "/icon-192.png", sizes: "192x192", type: "image/png" },
          { src: "/icon-512.png", sizes: "512x512", type: "image/png" },
          { src: "/favicon.svg", sizes: "any", type: "image/svg+xml" },
        ],
      },
    }),
  ],
    server: {
      // Same backend prefixes as frontend/nginx.conf so `npm run dev` matches docker.
      proxy: {
        "/api": api,
        "/auth": api,
        "/health": api,
        "/provision": api,
      },
    },
    build: {
      outDir: "dist",
    },
  };
});
