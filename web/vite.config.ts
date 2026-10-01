import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import wasm from "vite-plugin-wasm";
import topLevelAwait from "vite-plugin-top-level-await";

// The Codama-generated error module (`web/src/generated/zk_pool/src/generated/errors/zkPool.ts`)
// references `process.env.NODE_ENV`. That global does not exist in the browser, so Vite
// must define it at build time.
//
// `@noir-lang/acvm_js` and `@noir-lang/noirc_abi` are wasm-pack-generated modules that
// use `import ... from "*.wasm"` and top-level await. Vite cannot process them without
// `vite-plugin-wasm` and `vite-plugin-top-level-await`.
export default defineConfig({
  plugins: [vue(), wasm(), topLevelAwait()],
  define: {
    "process.env.NODE_ENV": JSON.stringify(process.env.NODE_ENV ?? "development"),
  },
  optimizeDeps: {
    exclude: ["@noir-lang/noir_js", "@noir-lang/acvm_js", "@noir-lang/noirc_abi"],
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
