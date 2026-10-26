import { expect, test, type Locator, type Page } from "@playwright/test";
import { TAURI_MOCK } from "./tauri-mock";

test.beforeEach(async ({ page }) => {
  await page.addInitScript(TAURI_MOCK);
  await page.goto("/");
  await expect(page.getByRole("toolbar", { name: "Dock" })).toBeVisible();
});

function scaleOf(locator: Locator): Promise<number> {
  return locator.evaluate((element) => {
    const transform = getComputedStyle(element).transform;
    const match = transform.match(/matrix\(([^)]+)\)/);
    return match ? Number(match[1].split(",")[0]) : 1;
  });
}

test("renders idle, running, minimized, and disabled tiles", async ({ page }) => {
  await expect(page.getByRole("button", { name: "Idle App", exact: true })).toBeVisible();

  await expect(
    page.getByRole("button", { name: "Running App, running", exact: true }),
  ).toBeVisible();

  const minimized = page.getByRole("button", {
    name: "Minimized App, running",
    exact: true,
  });
  await expect(minimized).toBeVisible();
  await expect(minimized.getByLabel("2 minimized")).toHaveText("2");

  await expect(page.getByRole("button", { name: "Broken App", exact: true })).toBeDisabled();
});

test("clicking a tile records launch_item for that tile", async ({ page }) => {
  await page.getByRole("button", { name: "Idle App", exact: true }).click();

  await expect
    .poll(() =>
      page.evaluate(() => {
        const calls = (window as unknown as { __MOCK_CALLS__: Array<{ cmd: string }> })
          .__MOCK_CALLS__;
        return calls.find((call) => call.cmd === "launch_item") ?? null;
      }),
    )
    .toEqual({ cmd: "launch_item", args: { id: "idle-app" } });
});

test("hovering a tile magnifies it and leaving restores it", async ({ page }) => {
  // Rust sizes the dock window to exactly the pill width (window_geom), while
  // the browser harness is full-page and centers the pill. Match the production
  // window width so the dock's pointer coordinate and its icon centers share an
  // origin. In a wider window, pointer X is measured against .dock-stage while
  // the icon centers are pill-local, so the magnify peak shifts right.
  const pillBox = await page.getByRole("toolbar", { name: "Dock" }).boundingBox();
  if (!pillBox) throw new Error("dock pill has no bounding box");
  await page.setViewportSize({ width: Math.ceil(pillBox.width), height: 720 });

  const tile = page.getByRole("button", { name: "Idle App", exact: true });

  await tile.hover();
  await expect.poll(() => scaleOf(tile), { timeout: 3_000 }).toBeGreaterThan(1.01);

  await page.mouse.move(10, 10);
  await expect.poll(() => scaleOf(tile), { timeout: 3_000 }).toBeLessThan(1.01);
});

test("tooltip appears only after a dwell and shows the app name", async ({ page }) => {
  const tile = page.getByRole("button", { name: "Idle App", exact: true });

  // Timestamp the dwell from the actual pointerenter, not from test start, so
  // the delay assertion is not sensitive to how long Playwright takes to move.
  await page.evaluate(() => {
    const state = window as unknown as {
      __HOVER_AT__?: number;
      __TOOLTIP_AT__?: number;
    };
    state.__HOVER_AT__ = undefined;
    state.__TOOLTIP_AT__ = undefined;
    const button = document.querySelector('button[aria-label="Idle App"]');
    button?.addEventListener(
      "mouseenter",
      () => {
        state.__HOVER_AT__ = performance.now();
      },
      { once: true },
    );
    const observer = new MutationObserver(() => {
      if (state.__TOOLTIP_AT__ === undefined && document.querySelector('[role="tooltip"]')) {
        state.__TOOLTIP_AT__ = performance.now();
        observer.disconnect();
      }
    });
    observer.observe(document.body, { childList: true, subtree: true });
  });

  const box = await tile.boundingBox();
  if (!box) throw new Error("idle tile has no bounding box");
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);

  await expect(page.getByRole("tooltip")).toHaveCount(0);
  await expect(page.getByRole("tooltip")).toHaveText("Idle App", { timeout: 3_000 });

  const delay = await page.evaluate(
    () =>
      (window as unknown as { __TOOLTIP_AT__: number }).__TOOLTIP_AT__ -
      (window as unknown as { __HOVER_AT__: number }).__HOVER_AT__,
  );
  expect(delay).toBeGreaterThan(400);
  expect(delay).toBeLessThan(2_000);
});

test("a dock://state push re-renders without reloading", async ({ page }) => {
  // Only emit once the app has actually subscribed.
  await expect
    .poll(() =>
      page.evaluate(() => {
        const calls = (
          window as unknown as {
            __MOCK_CALLS__: Array<{ cmd: string; args: { event?: string } }>;
          }
        ).__MOCK_CALLS__;
        return calls.some(
          (call) => call.cmd === "plugin:event|listen" && call.args.event === "dock://state",
        );
      }),
    )
    .toBe(true);

  await page.evaluate(() => {
    const mock = window as unknown as {
      __NO_RELOAD__: boolean;
      __MOCK_DOCK_STATE__: { items: Array<{ id: string; running: boolean }> };
      __MOCK_EMIT__: (event: string, payload: unknown) => void;
    };
    mock.__NO_RELOAD__ = true;
    const next = JSON.parse(JSON.stringify(mock.__MOCK_DOCK_STATE__));
    next.items.find((item) => item.id === "idle-app").running = true;
    mock.__MOCK_EMIT__("dock://state", next);
  });

  await expect(
    page.getByRole("button", { name: "Idle App, running", exact: true }),
  ).toBeVisible();
  expect(
    await page.evaluate(
      () => (window as unknown as { __NO_RELOAD__: boolean }).__NO_RELOAD__,
    ),
  ).toBe(true);
});
