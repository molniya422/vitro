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
  await expect(page.getByRole("img", { name: "Dock preview" })).toBeVisible();
});

test("settings exposes every section through tabs", async ({ page }) => {
  await openSettings(page);

  await page.getByRole("tab", { name: "Behaviour" }).click();
  await expect(page.getByRole("slider", { name: /magnify peak/i })).toBeVisible();
  await expect(
    page.getByRole("checkbox", { name: /auto-hide at the screen edge/i }),
  ).toBeVisible();

  await page.getByRole("tab", { name: "Apps" }).click();
  await expect(
    page.getByRole("searchbox", { name: "Search apps to add" }),
  ).toBeVisible();

  await page.getByRole("tab", { name: "Advanced" }).click();
  await expect(page.getByRole("button", { name: "Reset to defaults" })).toBeVisible();
});

test("apps can be removed and added from the settings page", async ({ page }) => {
  await openSettings(page);
  await page.getByRole("tab", { name: "Apps" }).click();
  await expect(page.getByRole("searchbox", { name: "Search apps to add" })).toBeVisible();

  await page.getByRole("button", { name: "Remove Files" }).click();
  await expect
    .poll(() =>
      page.evaluate(() => {
        const calls = (
          window as unknown as {
            __MOCK_CALLS__: Array<{ cmd: string; args: { id?: string } }>;
          }
        ).__MOCK_CALLS__;
        return calls.find((call) => call.cmd === "unpin_app")?.args.id ?? null;
      }),
    )
    .toBe("files-app");

  await page.getByRole("searchbox", { name: "Search apps to add" }).fill("Alpha");
  await page.getByRole("button", { name: "Add Alpha Player" }).click();
  await expect
    .poll(() =>
      page.evaluate(() => {
        const calls = (
          window as unknown as {
            __MOCK_CALLS__: Array<{ cmd: string; args: { id?: string } }>;
          }
        ).__MOCK_CALLS__;
        return calls.find((call) => call.cmd === "pin_app")?.args.id ?? null;
      }),
    )
    .toBe("cat-alpha");
});

test("saving settings invokes save_settings with the chosen icon size", async ({ page }) => {
  await openSettings(page);

  const slider = page.getByRole("slider", { name: /icon size/i });
  await expect(slider).toHaveValue("48");
  await slider.focus();
  await slider.press("ArrowRight");
  await expect(page.getByText("50 px")).toBeVisible();

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
              args: { settings?: { iconSize?: number } };
            }>;
          }
        ).__MOCK_CALLS__;
        const call = calls.find((entry) => entry.cmd === "save_settings");
        return call?.args.settings?.iconSize ?? null;
      }),
    )
    .toBe(50);
});

test("a saved icon size reaches the dock stage", async ({ page }) => {
  await openSettings(page);

  const slider = page.getByRole("slider", { name: /icon size/i });
  await slider.focus();
  await slider.press("ArrowRight");
  await expect(page.getByText("50 px")).toBeVisible();
  await page.getByRole("button", { name: "Save" }).click();
  await expect(page.getByText("Saved.", { exact: true })).toBeVisible();
  await page.getByRole("button", { name: "Close" }).click();

  await expect(page.getByRole("toolbar", { name: "Dock" })).toBeVisible();
  const size = await page
    .locator(".dock-stage")
    .evaluate((element) => getComputedStyle(element).getPropertyValue("--icon-size").trim());
  expect(size).toBe("50px");
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

  await expect(slider).toHaveValue("8");
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

test("dragging a launchpad app onto the dock strip pins it", async ({ page }) => {
  // The mock dock starts at its 10-item cap; free a slot first.
  await page.getByRole("button", { name: "Files", exact: true }).click({
    button: "right",
  });
  await page.getByRole("menuitem", { name: "Remove from Dock" }).click();
  await expect(
    page.getByRole("button", { name: "Files", exact: true }),
  ).toHaveCount(0);

  await openLaunchpad(page);
  const tile = page.getByRole("button", { name: "Alpha Player", exact: true });
  const box = await tile.boundingBox();
  if (!box) throw new Error("Alpha Player has no bounding box");

  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
  await page.mouse.down();
  await page.mouse.move(box.x + box.width / 2, 700, { steps: 12 });

  const strip = page.getByRole("group", { name: "Dock drop target" });
  await expect(strip).toBeVisible();
  const stripBox = await strip.boundingBox();
  if (!stripBox) throw new Error("dock strip has no bounding box");
  await page.mouse.move(
    stripBox.x + stripBox.width - 24,
    stripBox.y + stripBox.height / 2,
    { steps: 4 },
  );
  await page.mouse.up();

  await expect
    .poll(() =>
      page.evaluate(() => {
        const calls = (
          window as unknown as {
            __MOCK_CALLS__: Array<{ cmd: string; args: { id?: string } }>;
          }
        ).__MOCK_CALLS__;
        return calls.find((call) => call.cmd === "pin_app")?.args.id ?? null;
      }),
    )
    .toBe("cat-alpha");

  // Releasing over the strip is a drop, not a launch.
  expect(
    await page.evaluate(() => {
      const calls = (
        window as unknown as {
          __MOCK_CALLS__: Array<{ cmd: string; args: { id?: string } }>;
        }
      ).__MOCK_CALLS__;
      return calls.some(
        (call) => call.cmd === "launch_item" && call.args.id === "cat-alpha",
      );
    }),
  ).toBe(false);
});

test("the shelf tile opens an empty shelf", async ({ page }) => {
  await page.getByRole("button", { name: "Open Shelf" }).click();

  await expect(page.getByRole("heading", { name: "Shelf" })).toBeVisible();
  await expect(
    page.getByText("Drag files here to keep them handy."),
  ).toBeVisible();
});

test("a dropped file opens the shelf and can be opened and removed", async ({
  page,
}) => {
  // The drop listener is installed asynchronously after the first render.
  await expect
    .poll(() =>
      page.evaluate(() => {
        const calls = (
          window as unknown as {
            __MOCK_CALLS__: Array<{ cmd: string; args: { event?: string } }>;
          }
        ).__MOCK_CALLS__;
        return calls.some(
          (call) =>
            call.cmd === "plugin:event|listen" &&
            call.args.event === "tauri://drag-drop",
        );
      }),
    )
    .toBe(true);

  await page.evaluate(() => {
    (
      window as unknown as {
        __MOCK_EMIT__: (event: string, payload: unknown) => void;
      }
    ).__MOCK_EMIT__("tauri://drag-drop", {
      paths: ["C:\\Users\\Public\\report.pdf"],
      position: { x: 10, y: 10 },
    });
  });

  await expect(page.getByRole("heading", { name: "Shelf" })).toBeVisible();
  const item = page.getByRole("button", { name: "Open report.pdf" });
  await expect(item).toBeVisible();

  await item.click();
  await expect
    .poll(() =>
      page.evaluate(() => {
        const calls = (
          window as unknown as {
            __MOCK_CALLS__: Array<{ cmd: string; args: { path?: string } }>;
          }
        ).__MOCK_CALLS__;
        return calls.find((call) => call.cmd === "open_shelf_item")?.args
          .path ?? null;
      }),
    )
    .toBe("C:\\Users\\Public\\report.pdf");

  await page.getByRole("button", { name: "Remove report.pdf" }).click();
  await expect(item).toHaveCount(0);
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
