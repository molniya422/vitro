import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "./e2e",
  fullyParallel: false,
  workers: 1,
  // The first test after a cold Vite start pays the on-demand module compile;
  // everything after it is sub-second.
  timeout: 90_000,
  // Keep run artifacts inside node_modules so they never reach git.
  outputDir: "node_modules/.playwright-artifacts",
  reporter: "list",
  use: {
    baseURL: "http://localhost:1420",
    viewport: { width: 1280, height: 720 },
    // The first navigation after a cold Vite start waits for on-demand
    // dependency optimization; 30 s is not always enough on this machine.
    navigationTimeout: 60_000,
  },
  webServer: {
    command: "npm run dev",
    url: "http://localhost:1420",
    reuseExistingServer: true,
    timeout: 120_000,
  },
  projects: [{ name: "chromium", use: { browserName: "chromium" } }],
});
