import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { api } from "../api";
import type { DockItemView, SettingsView } from "../types";
import { DEFAULT_SETTINGS, validateSettings } from "./validate";
import { DockPreview } from "./preview";
import "./settings.css";

interface Props {
  onClose: () => void;
}

type Tab = "appearance" | "behaviour" | "apps" | "advanced";
type SaveStatus = "idle" | "saving" | "saved";

const DOCK_ITEM_CAP = 10;
const SAVE_DEBOUNCE_MS = 300;

const TABS: Array<{ id: Tab; label: string }> = [
  { id: "appearance", label: "Appearance" },
  { id: "behaviour", label: "Behaviour" },
  { id: "apps", label: "Apps" },
  { id: "advanced", label: "Advanced" },
];

interface RangeFieldProps {
  label: string;
  value: number;
  min: number;
  max: number;
  step?: number;
  format?: (value: number) => string;
  onChange: (value: number) => void;
}

function RangeField({
  label,
  value,
  min,
  max,
  step = 1,
  format = (next) => `${next} px`,
  onChange,
}: RangeFieldProps) {
  return (
    <label className="settings-field">
      <span className="settings-field-label">{label}</span>
      <input
        type="range"
        aria-label={label}
        min={min}
        max={max}
        step={step}
        value={value}
        onChange={(event) => onChange(Number(event.target.value))}
      />
      <output>{format(value)}</output>
    </label>
  );
}

interface CheckFieldProps {
  label: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
}

function CheckField({ label, checked, onChange }: CheckFieldProps) {
  return (
    <label className="settings-check">
      <input
        type="checkbox"
        checked={checked}
        onChange={(event) => onChange(event.target.checked)}
      />
      {label}
    </label>
  );
}

export function SettingsPanel({ onClose }: Props) {
  const [settings, setSettings] = useState<SettingsView | null>(null);
  const [pinned, setPinned] = useState<DockItemView[] | null>(null);
  const [catalog, setCatalog] = useState<DockItemView[] | null>(null);
  const [tab, setTab] = useState<Tab>("appearance");
  const [query, setQuery] = useState("");
  const [errors, setErrors] = useState<string[]>([]);
  const [appsError, setAppsError] = useState<string | null>(null);
  const [status, setStatus] = useState<SaveStatus>("idle");
  const [confirmReset, setConfirmReset] = useState(false);

  const firstLoad = useRef(true);
  const dirty = useRef(false);
  const saveSeq = useRef(0);
  const saveTimer = useRef<number | null>(null);

  useEffect(() => {
    Promise.all([api.getSettings(), api.getDockState()])
      .then(([view, state]) => {
        setSettings(view);
        setPinned(state.items);
      })
      .catch((error) => setErrors([String(error)]));
  }, []);

  useEffect(() => {
    if (tab !== "apps" || catalog !== null) return;
    let cancelled = false;
    api
      .getCatalog()
      .then((items) => {
        if (!cancelled) setCatalog(items);
      })
      .catch((error) => {
        if (!cancelled) setAppsError(String(error));
      });
    return () => {
      cancelled = true;
    };
  }, [catalog, tab]);

  const persist = useCallback(async (value: SettingsView): Promise<boolean> => {
    const result = validateSettings(value);
    if (!result.ok) {
      setErrors(result.errors);
      setStatus("idle");
      return false;
    }
    setErrors([]);
    const seq = ++saveSeq.current;
    setStatus("saving");
    try {
      await api.saveSettings(value);
      if (seq === saveSeq.current) setStatus("saved");
      return true;
    } catch (error) {
      if (seq === saveSeq.current) {
        setErrors([String(error)]);
        setStatus("idle");
      }
      return false;
    }
  }, []);

  // Live apply: every change is persisted after a short idle. The preview
  // updates immediately; the dock and window geometry follow the backend push.
  useEffect(() => {
    if (!settings) return;
    if (firstLoad.current) {
      firstLoad.current = false;
      return;
    }
    // The initial load, a StrictMode re-mount, and the reset button all set
    // state without marking it dirty. Only user edits are auto-persisted.
    if (!dirty.current) return;
    dirty.current = false;
    if (saveTimer.current !== null) window.clearTimeout(saveTimer.current);
    saveTimer.current = window.setTimeout(() => {
      saveTimer.current = null;
      void persist(settings);
    }, SAVE_DEBOUNCE_MS);
    return () => {
      if (saveTimer.current !== null) {
        window.clearTimeout(saveTimer.current);
        saveTimer.current = null;
      }
    };
  }, [settings, persist]);

  const update = useCallback((patch: Partial<SettingsView>) => {
    dirty.current = true;
    setSettings((current) => (current ? { ...current, ...patch } : current));
    setStatus("idle");
  }, []);

  const saveNow = useCallback(async () => {
    if (!settings) return;
    if (saveTimer.current !== null) {
      window.clearTimeout(saveTimer.current);
      saveTimer.current = null;
    }
    await persist(settings);
  }, [persist, settings]);

  const close = useCallback(async () => {
    if (!settings) {
      onClose();
      return;
    }
    if (saveTimer.current !== null) {
      window.clearTimeout(saveTimer.current);
      saveTimer.current = null;
    }
    const ok = await persist(settings);
    if (ok) onClose();
  }, [onClose, persist, settings]);

  const refreshPinned = useCallback(async () => {
    const state = await api.getDockState();
    setPinned(state.items);
  }, []);

  const addApp = useCallback(
    async (id: string) => {
      setAppsError(null);
      try {
        await api.pinApp(id);
        await refreshPinned();
      } catch (error) {
        setAppsError(String(error));
      }
    },
    [refreshPinned],
  );

  const removeApp = useCallback(
    async (id: string) => {
      setAppsError(null);
      try {
        await api.unpinApp(id);
        await refreshPinned();
      } catch (error) {
        setAppsError(String(error));
      }
    },
    [refreshPinned],
  );

  const moveApp = useCallback(
    async (index: number, delta: number) => {
      if (!pinned) return;
      const target = index + delta;
      if (target < 0 || target >= pinned.length) return;
      const next = [...pinned];
      [next[index], next[target]] = [next[target], next[index]];
      setAppsError(null);
      try {
        await api.reorderPins(next.map((item) => item.id));
        setPinned(next);
      } catch (error) {
        setAppsError(String(error));
      }
    },
    [pinned],
  );

  const pinnedIds = useMemo(
    () => new Set((pinned ?? []).map((item) => item.id)),
    [pinned],
  );
  const filteredCatalog = useMemo(() => {
    const needle = query.trim().toLowerCase();
    return (catalog ?? []).filter(
      (item) =>
        !item.disabledReason &&
        (needle === "" || item.name.toLowerCase().includes(needle)),
    );
  }, [catalog, query]);
  const dockFull = (pinned?.length ?? 0) >= DOCK_ITEM_CAP;

  return (
    <section className="settings-panel" aria-label="Settings">
      <header>
        <h2>Settings</h2>
        <div className="settings-header-actions">
          <span className="settings-status" role="status">
            {status === "saving" ? "Saving…" : status === "saved" ? "Saved." : ""}
          </span>
          <button type="button" className="settings-save" onClick={() => void saveNow()}>
            Save
          </button>
          <button type="button" onClick={() => void close()}>
            Close
          </button>
        </div>
      </header>

      <div className="settings-tabs" role="tablist" aria-label="Settings sections">
        {TABS.map((entry) => (
          <button
            key={entry.id}
            type="button"
            role="tab"
            id={`settings-tab-${entry.id}`}
            aria-controls={`settings-panel-${entry.id}`}
            aria-selected={tab === entry.id}
            className={tab === entry.id ? "is-active" : ""}
            onClick={() => setTab(entry.id)}
          >
            {entry.label}
          </button>
        ))}
      </div>

      {errors.length > 0 && (
        <ul role="alert" className="settings-errors">
          {errors.map((error) => (
            <li key={error}>{error}</li>
          ))}
        </ul>
      )}

      {!settings ? (
        <p className="settings-loading" aria-busy="true">
          Loading settings…
        </p>
      ) : (
        <div
          className="settings-body"
          role="tabpanel"
          id={`settings-panel-${tab}`}
          aria-labelledby={`settings-tab-${tab}`}
        >
          {tab === "appearance" && (
            <div className="settings-tab-content">
              <div className="settings-grid">
                <RangeField
                  label="Icon size"
                  value={settings.iconSize}
                  min={32}
                  max={80}
                  step={2}
                  onChange={(iconSize) => update({ iconSize })}
                />
                <RangeField
                  label="Icon spacing"
                  value={settings.iconGap}
                  min={0}
                  max={24}
                  onChange={(iconGap) => update({ iconGap })}
                />
                <RangeField
                  label="Horizontal padding"
                  value={settings.paddingH}
                  min={4}
                  max={32}
                  onChange={(paddingH) => update({ paddingH })}
                />
                <RangeField
                  label="Vertical padding"
                  value={settings.paddingV}
                  min={2}
                  max={24}
                  onChange={(paddingV) => update({ paddingV })}
                />
                <RangeField
                  label="Bar corner radius"
                  value={settings.pillRadius}
                  min={0}
                  max={64}
                  onChange={(pillRadius) => update({ pillRadius })}
                />
                <RangeField
                  label="Glass opacity"
                  value={settings.fillOpacity}
                  min={0.15}
                  max={1}
                  step={0.01}
                  format={(value) => `${Math.round(value * 100)}%`}
                  onChange={(fillOpacity) => update({ fillOpacity })}
                />
                <label className="settings-field">
                  <span className="settings-field-label">Glass tint</span>
                  <input
                    type="color"
                    aria-label="Glass tint"
                    value={settings.tint}
                    onChange={(event) => update({ tint: event.target.value })}
                  />
                  <output>{settings.tint}</output>
                </label>
                <RangeField
                  label="Icon corner radius"
                  value={settings.iconRadius}
                  min={0}
                  max={50}
                  format={(value) => `${value}%`}
                  onChange={(iconRadius) => update({ iconRadius })}
                />
                <RangeField
                  label="Reflection strength"
                  value={settings.reflectionStrength}
                  min={0}
                  max={1}
                  step={0.01}
                  format={(value) => `${Math.round(value * 100)}%`}
                  onChange={(reflectionStrength) => update({ reflectionStrength })}
                />
              </div>
              <div className="settings-checks">
                <CheckField
                  label="Show reflection"
                  checked={settings.reflection}
                  onChange={(reflection) => update({ reflection })}
                />
              </div>
              <DockPreview settings={settings} />
            </div>
          )}

          {tab === "behaviour" && (
            <div className="settings-tab-content">
              <div className="settings-checks">
                <CheckField
                  label="Magnification"
                  checked={settings.magnify}
                  onChange={(magnify) => update({ magnify })}
                />
                <CheckField
                  label="Reduced motion"
                  checked={settings.reducedMotion}
                  onChange={(reducedMotion) => update({ reducedMotion })}
                />
                <CheckField
                  label="Show tooltips"
                  checked={settings.tooltips}
                  onChange={(tooltips) => update({ tooltips })}
                />
                <CheckField
                  label="Show running indicators"
                  checked={settings.showRunningDots}
                  onChange={(showRunningDots) => update({ showRunningDots })}
                />
                <CheckField
                  label="Show minimized counts"
                  checked={settings.showBadges}
                  onChange={(showBadges) => update({ showBadges })}
                />
              </div>
              <div className="settings-grid">
                <RangeField
                  label="Magnify peak"
                  value={settings.magnifyPeak}
                  min={1}
                  max={2}
                  step={0.05}
                  format={(value) => `${value.toFixed(2)}×`}
                  onChange={(magnifyPeak) => update({ magnifyPeak })}
                />
                <RangeField
                  label="Magnify reach"
                  value={settings.magnifyRadiusIcons}
                  min={1}
                  max={5}
                  step={0.1}
                  format={(value) => `${value.toFixed(1)} icons`}
                  onChange={(magnifyRadiusIcons) => update({ magnifyRadiusIcons })}
                />
                <RangeField
                  label="Tooltip delay"
                  value={settings.tooltipDelayMs}
                  min={0}
                  max={2000}
                  step={50}
                  format={(value) => `${value} ms`}
                  onChange={(tooltipDelayMs) => update({ tooltipDelayMs })}
                />
                <label className="settings-field">
                  <span className="settings-field-label">Motion profile</span>
                  <select
                    aria-label="Motion profile"
                    value={settings.profile}
                    onChange={(event) =>
                      update({
                        profile: event.target.value as SettingsView["profile"],
                      })
                    }
                  >
                    <option value="vitro">Elastic</option>
                    <option value="macos">Restrained</option>
                  </select>
                </label>
              </div>

              <h3 className="settings-section-title">Auto-hide</h3>
              <div className="settings-checks">
                <CheckField
                  label="Auto-hide at the screen edge"
                  checked={settings.autohide}
                  onChange={(autohide) => update({ autohide })}
                />
                <CheckField
                  label="Hide over fullscreen apps"
                  checked={settings.yieldFullscreen}
                  onChange={(yieldFullscreen) => update({ yieldFullscreen })}
                />
              </div>
              <div className="settings-grid">
                <RangeField
                  label="Reveal delay"
                  value={settings.showDelayMs}
                  min={0}
                  max={1000}
                  step={50}
                  format={(value) => `${value} ms`}
                  onChange={(showDelayMs) => update({ showDelayMs })}
                />
                <RangeField
                  label="Hide delay"
                  value={settings.hideDelayMs}
                  min={100}
                  max={3000}
                  step={50}
                  format={(value) => `${value} ms`}
                  onChange={(hideDelayMs) => update({ hideDelayMs })}
                />
              </div>

              <h3 className="settings-section-title">Dock entries</h3>
              <div className="settings-checks">
                <CheckField
                  label="Show Launchpad tile"
                  checked={settings.showLaunchpadEntry}
                  onChange={(showLaunchpadEntry) => update({ showLaunchpadEntry })}
                />
                <CheckField
                  label="Show monitor tile"
                  checked={settings.showMonitorEntry}
                  onChange={(showMonitorEntry) => update({ showMonitorEntry })}
                />
                <CheckField
                  label="Show shelf tile"
                  checked={settings.showShelfEntry}
                  onChange={(showShelfEntry) => update({ showShelfEntry })}
                />
              </div>
            </div>
          )}

          {tab === "apps" && (
            <div className="settings-tab-content">
              <p className="settings-hint">
                {pinned?.length ?? 0} of {DOCK_ITEM_CAP} dock slots used. The dock
                shows up to {DOCK_ITEM_CAP} apps.
              </p>
              {appsError && (
                <p className="settings-apps-error" role="alert">
                  {appsError}
                </p>
              )}
              <div className="settings-apps">
                <section aria-label="Pinned apps">
                  <h3 className="settings-section-title">In the dock</h3>
                  {pinned === null ? (
                    <p className="settings-hint">Loading…</p>
                  ) : pinned.length === 0 ? (
                    <p className="settings-hint">No pinned apps.</p>
                  ) : (
                    <ul className="settings-app-list">
                      {pinned.map((item, index) => (
                        <li key={item.id}>
                          <span className="settings-app-name">{item.name}</span>
                          <span className="settings-app-actions">
                            <button
                              type="button"
                              aria-label={`Move ${item.name} up`}
                              disabled={index === 0}
                              onClick={() => void moveApp(index, -1)}
                            >
                              ↑
                            </button>
                            <button
                              type="button"
                              aria-label={`Move ${item.name} down`}
                              disabled={index === pinned.length - 1}
                              onClick={() => void moveApp(index, 1)}
                            >
                              ↓
                            </button>
                            <button
                              type="button"
                              aria-label={`Remove ${item.name}`}
                              disabled={item.kind === "system"}
                              onClick={() => void removeApp(item.id)}
                            >
                              Remove
                            </button>
                          </span>
                        </li>
                      ))}
                    </ul>
                  )}
                </section>

                <section aria-label="Add apps">
                  <h3 className="settings-section-title">Add an app</h3>
                  <input
                    type="search"
                    className="settings-search"
                    aria-label="Search apps to add"
                    placeholder="Search applications"
                    value={query}
                    onChange={(event) => setQuery(event.target.value)}
                  />
                  {catalog === null ? (
                    <p className="settings-hint" aria-busy="true">
                      Loading applications…
                    </p>
                  ) : (
                    <ul className="settings-app-list">
                      {filteredCatalog.map((item) => {
                        const isPinned = pinnedIds.has(item.id);
                        return (
                          <li key={item.id}>
                            <span className="settings-app-name">{item.name}</span>
                            <button
                              type="button"
                              aria-label={`Add ${item.name}`}
                              disabled={isPinned || dockFull}
                              onClick={() => void addApp(item.id)}
                            >
                              {isPinned ? "Added" : "Add"}
                            </button>
                          </li>
                        );
                      })}
                      {filteredCatalog.length === 0 && (
                        <li className="settings-hint">No applications match.</li>
                      )}
                    </ul>
                  )}
                </section>
              </div>
            </div>
          )}

          {tab === "advanced" && (
            <div className="settings-tab-content">
              <h3 className="settings-section-title">Reset</h3>
              <p className="settings-hint">
                Restores every appearance and behaviour value to its default. Pinned
                apps are kept.
              </p>
              {confirmReset ? (
                <div className="settings-reset-actions">
                  <button
                    type="button"
                    onClick={() => {
                      dirty.current = true;
                      setSettings(DEFAULT_SETTINGS);
                      setConfirmReset(false);
                      setStatus("idle");
                    }}
                  >
                    Confirm reset
                  </button>
                  <button type="button" onClick={() => setConfirmReset(false)}>
                    Cancel
                  </button>
                </div>
              ) : (
                <button type="button" onClick={() => setConfirmReset(true)}>
                  Reset to defaults
                </button>
              )}

              <h3 className="settings-section-title">About</h3>
              <p className="settings-hint">
                Settings file: %APPDATA%\DockApp\settings.json. Writes are atomic
                and keep a .bak copy.
              </p>
              <p className="settings-hint">
                The glass is painted (opacity + tint + grain). Real desktop blur is
                not applied yet, so transparency affects the material colour, not
                the wallpaper behind the window.
              </p>
            </div>
          )}
        </div>
      )}
    </section>
  );
}
