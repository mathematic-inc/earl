import { fileURLToPath, URL } from "node:url";

import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import { defineConfig, type Plugin } from "vite";

const EARL_TOKEN_RE = /earl-token"\s+content="([^"]+)"/v;

function earlTokenPlugin(): Plugin {
  const target = process.env.EARL_API_URL || "http://localhost:3000";
  let cachedToken: string | null = null;

  return {
    name: "earl-token",
    apply: "serve",
    async transformIndexHtml() {
      if (!cachedToken) {
        try {
          const res = await fetch(target);
          const html = await res.text();
          const token = html.match(EARL_TOKEN_RE)?.[1];
          if (token) {
            cachedToken = token;
          }
        } catch {
          // earl backend not running
        }
      }
      if (cachedToken) {
        return [
          {
            tag: "meta",
            attrs: { name: "earl-token", content: cachedToken },
            injectTo: "head",
          },
        ];
      }
      return [];
    },
  };
}

export default defineConfig({
  plugins: [react(), tailwindcss(), earlTokenPlugin()],
  server: {
    proxy: {
      "/api": process.env.EARL_API_URL || "http://localhost:3000",
    },
  },
  resolve: {
    alias: {
      "@": fileURLToPath(new URL("./src", import.meta.url)),
      "./files": fileURLToPath(
        new URL("./node_modules/@fontsource-variable/inter/files", import.meta.url),
      ),
    },
  },
});
