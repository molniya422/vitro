import { useEffect, useRef, useState } from "react";
import { api } from "../api";
import type { MonitorSample } from "../types";
import { formatBytes, formatPercent } from "./format";
import { Sparkline } from "./Sparkline";
import "./monitor.css";

interface Props {
  onClose: () => void;
}

const SAMPLE_INTERVAL_MS = 1000;
const HISTORY = 60;

export function MonitorPanel({ onClose }: Props) {
  const [sample, setSample] = useState<MonitorSample | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [cpuHistory, setCpuHistory] = useState<number[]>([]);
  const timer = useRef<number | null>(null);

  useEffect(() => {
    let cancelled = false;

    const tick = async () => {
      try {
        const next = await api.sampleMonitor();
        if (cancelled) return;
        setError(null);
        setSample(next);
        setCpuHistory((history) => [...history, next.cpu].slice(-HISTORY));
      } catch (e) {
        if (!cancelled) setError(String(e));
      }
    };

    void tick();
    timer.current = window.setInterval(() => void tick(), SAMPLE_INTERVAL_MS);
    return () => {
      cancelled = true;
      if (timer.current !== null) window.clearInterval(timer.current);
    };
  }, []);

  return (
    <section className="monitor-panel" aria-label="System monitor">
      <header>
        <h2>System</h2>
        <button type="button" onClick={onClose}>
          Close
        </button>
      </header>

      {error && <p role="alert">{error}</p>}

      {sample && (
        <dl>
          <div className="monitor-row">
            <dt>CPU</dt>
            <dd>{formatPercent(sample.cpu)}</dd>
          </div>
          <div className="monitor-sparkline">
            <Sparkline values={cpuHistory} />
          </div>
          <div className="monitor-row">
            <dt>Memory</dt>
            <dd>
              {formatBytes(sample.memUsed)} / {formatBytes(sample.memTotal)}
            </dd>
          </div>
          <div className="monitor-row">
            <dt>Disk</dt>
            <dd>
              {formatBytes(sample.diskUsed)} / {formatBytes(sample.diskTotal)}
            </dd>
          </div>
          <div className="monitor-row">
            <dt>Network</dt>
            <dd>
              ↓ {formatBytes(sample.netRx)} · ↑ {formatBytes(sample.netTx)}
            </dd>
          </div>
        </dl>
      )}
    </section>
  );
}
