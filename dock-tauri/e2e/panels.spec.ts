import { expect, test, type Page } from "@playwright/test";
import { TAURI_MOCK } from "./tauri-mock";

test.beforeEach(async ({ page }) => {
  await page.addInitScript(TAURI_MOCK);
  await page.goto("/");
  await expect(page.getByRole("toolbar", { name: "Dock" })).toBeVisible();
});

async function openSettings(page: Page) {
  await page.getByRole("button", { name: "Dock settings" }).click();
  await expect(page.getByRole("heading", { name: "Settings" })).toBeVisible();
}

async function openLaunchpad(page: Page) {
  await page.getByRole("button", { name: "Open Launchpad" }).click();
  await expect(page.getByRole("searchbox", { name: "Search applications" })).toBeVisible();
}

async function openMonitor(page: Page) {
  await page.getByRole("button", { name: "Open system monitor" }).click();
  await expect(page.getByRole("region", { name: "System monitor" })).toBeVisible();
}

test("a single click on the control tile opens Settings", async ({ page }) => {
  await openSettings(page);
  await expect(page.getByRole("slider", { name: /icon size/i })).toBeVisible();
});

test("saving settings invokes save_settings with the chosen icon size", async ({ page }) => {
  await openSettings(page);

  const slider = page.getByRole("slider", { name: /icon size/i });
  await expect(slider).toHaveValue("48");
  await slider.focus();
  await slider.press("ArrowRight");
  await expect(page.getByText("52 px")).toBeVisible();

  await page.getByRole("button", { name: "Save" }).click();
  // <output> also has the implicit `status` role, so target the message text.
  await expect(page.getByText("Saved.", { exact: true })).toBeVisible();

  await expect
    .poll(() =>
      page.evaluate(() => {
        const calls = (
          window as unknown as {
            __MOCK_CALLS__: Array<{
              cmd: string;
              args: { settings?: { icon_size?: number } };
            }>;
          }
        ).__MOCK_CALLS__;
        const call = calls.find((entry) => entry.cmd === "save_settings");
        return call?.args.settings?.icon_size ?? null;
      }),
    )
    .toBe(52);
});

test("client-side validation rejects an out-of-range icon size", async ({ page }) => {
  await openSettings(page);

  const slider = page.getByRole("slider", { name: /icon size/i });
  await expect(slider).toHaveValue("48");

  // A range input clamps user input to [min, max], so widen the constraint and
  // push 8 through the real React onChange path. This is the only way to reach
  // the component's invalid branch from the rendered UI; validateSettings'
  // rejection of 8 and 200 is also asserted in src/settings/validate.test.ts.
  await slider.evaluate((element) => {
    const input = element as HTMLInputElement;
    input.min = "0";
    const setter = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")?.set;
    if (!setter) throw new Error("no native input value setter");
    setter.call(input, "8");
    input.dispatchEvent(new Event("input", { bubbles: true }));
  });

  await expect(page.getByText("8 px")).toBeVisible();
  await page.getByRole("button", { name: "Save" }).click();

  await expect(page.getByRole("alert")).toContainText("iconSize");
  await expect(page.getByText("Saved.", { exact: true })).toHaveCount(0);
  expect(
    await page.evaluate(() => {
      const calls = (window as unknown as { __MOCK_CALLS__: Array<{ cmd: string }> })
        .__MOCK_CALLS__;
      return calls.some((call) => call.cmd === "save_settings");
    }),
  ).toBe(false);
});

test("launchpad search filters the grid to the matching app", async ({ page }) => {
  await openLaunchpad(page);

  await page.getByRole("searchbox", { name: "Search applications" }).fill("bravo");

  await expect(
    page.getByRole("button", { name: "Bravo Editor", exact: true }),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Alpha Player", exact: true }),
  ).toHaveCount(0);
  await expect(
    page.getByRole("button", { name: "Charlie Notes", exact: true }),
  ).toHaveCount(0);
});

test("launchpad shows an empty state when nothing matches", async ({ page }) => {
  await openLaunchpad(page);

  await page.getByRole("searchbox", { name: "Search applications" }).fill("zzzz");

  await expect(page.getByText(/No applications match/)).toHaveText(
    "No applications match “zzzz”.",
  );
});

test("monitor renders the fixture sample", async ({ page }) => {
  await openMonitor(page);

  await expect(page.getByText("42.5%", { exact: true })).toBeVisible();
  await expect(page.getByText("6.0 GB / 16.0 GB")).toBeVisible();
});

test("monitor surfaces a sample_monitor failure while staying rendered", async ({ page }) => {
  await page.evaluate(() => {
    (window as unknown as { __MOCK_FAIL__: (cmd: string) => void }).__MOCK_FAIL__(
      "sample_monitor",
    );
  });

  await openMonitor(page);

  await expect(page.getByRole("alert")).toContainText("mock failure: sample_monitor");
  await expect(page.getByRole("region", { name: "System monitor" })).toBeVisible();
});
