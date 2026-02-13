import { defineConfig } from "vitest/config";
import path from "path";

export default defineConfig({
  test: {
    globals: true,
    environment: "node",
  },
  resolve: {
    alias: {
      "@regen/core": path.resolve(__dirname, "packages/core/src"),
      "@regen/plugin-ledger-mcp": path.resolve(__dirname, "packages/plugin-ledger-mcp/src"),
      "@regen/plugin-koi-mcp": path.resolve(__dirname, "packages/plugin-koi-mcp/src"),
    },
  },
});
