import { useEffect, useState } from "react";
import { api } from "../api";
import { validateSettings, type SettingsInput } from "./validate";

interface Props {
  onClose: () => void;
}

const DEFAULTS: SettingsInput = {
  iconSize: 48,
  magnify: true,
  autohide: false,
  reducedMotion: false,
  profile: "vitro",
  showMonitor: false,
};

export function SettingsPanel({ onClose }: Props) {
  const [form, setForm] = useState<SettingsInput>(DEFAULTS);
  const [errors, setErrors] = useState<string[]>([]);
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    api
      .getSettings()
      .then((raw) => {
        const mapped: SettingsInput = {
          iconSize: Number(raw.icon_size ?? raw.iconSize ?? DEFAULTS.iconSize),
          magnify: Boolean(raw.magnify ?? DEFAULTS.magnify),
          autohide: Boolean(raw.autohide ?? DEFAULTS.autohide),
          reducedMotion: Boolean(raw.reduced_motion ?? raw.reducedMotion),
          profile: String(raw.profile ?? DEFAULTS.profile),
          showMonitor: Boolean(raw.show_monitor ?? raw.showMonitor),
        };
        setForm(mapped);
      })
      .catch((e) => setErrors([String(e)]));
  }, []);

  const save = async () => {
    const result = validateSettings(form);
    if (!result.ok) {
      setErrors(result.errors);
      setSaved(false);
      return;
    }
    setErrors([]);
    try {
      await api.saveSettings({
        schema_version: 1,
        icon_size: form.iconSize,
        magnify: form.magnify,
        autohide: form.autohide,
        reduced_motion: form.reducedMotion,
        profile: form.profile,
        show_monitor: form.showMonitor,
        pinned: [],
      });
      setSaved(true);
    } catch (e) {
      setErrors([String(e)]);
    }
  };

  return (
    <section className="settings-panel" aria-label="Settings">
      <header>
        <h2>Settings</h2>
        <button type="button" onClick={onClose}>
          Close
        </button>
      </header>

      <label>
        Icon size
        <input
          type="range"
          min={16}
          max={128}
          step={4}
          value={form.iconSize}
          onChange={(e) => setForm({ ...form, iconSize: Number(e.target.value) })}
        />
        <output>{form.iconSize} px</output>
      </label>

      <label>
        <input
          type="checkbox"
          checked={form.magnify}
          onChange={(e) => setForm({ ...form, magnify: e.target.checked })}
        />
        Magnification
      </label>

      <label>
        <input
          type="checkbox"
          checked={form.autohide}
          onChange={(e) => setForm({ ...form, autohide: e.target.checked })}
        />
        Auto-hide when the pointer leaves
      </label>

      <label>
        <input
          type="checkbox"
          checked={form.reducedMotion}
          onChange={(e) => setForm({ ...form, reducedMotion: e.target.checked })}
        />
        Reduced motion
      </label>

      <label>
        Motion profile
        <select
          value={form.profile}
          onChange={(e) => setForm({ ...form, profile: e.target.value })}
        >
          <option value="vitro">Elastic</option>
          <option value="macos">Restrained</option>
        </select>
      </label>

      <label>
        <input
          type="checkbox"
          checked={form.showMonitor}
          onChange={(e) => setForm({ ...form, showMonitor: e.target.checked })}
        />
        Show system monitor
      </label>

      {errors.length > 0 && (
        <ul role="alert">
          {errors.map((error) => (
            <li key={error}>{error}</li>
          ))}
        </ul>
      )}

      <button type="button" onClick={() => void save()}>
        Save
      </button>
      {saved && <p role="status">Saved.</p>}
    </section>
  );
}
