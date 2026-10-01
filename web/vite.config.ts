import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";

// The Codama-generated error module (`web/src/generated/zk_pool/src/generated/errors/zkPool.ts`)
// references `process.env.NODE_ENV`. That global does not exist in the browser, so Vite
// must define it at build time.
export default defineConfig({
  plugins: [vue()],
  define: {
    "process.env.NODE_ENV": JSON.stringify(process.env.NODE_ENV ?? "development"),
  },
  css: {
    preprocessorOptions: {
      scss: { api: "modern-compiler" },
    },
  },
  server: {
    host: "0.0.0.0",
    port: 5173,
    proxy: { "/api": "http://localhost:4001" },
  },
});
