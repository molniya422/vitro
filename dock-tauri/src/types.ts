export type Edge = "bottom";

export type MotionProfile = "macos" | "vitro";

export type ItemKind = "app" | "separator" | "system";

export interface DockItemView {
  id: string;
  kind: ItemKind;
  name: string;
  icon: string | null;
  running: boolean;
  minimized: number;
  disabledReason: string | null;
}

export interface DockState {
  items: DockItemView[];
  edge: Edge;
  autohide: boolean;
  magnify: boolean;
  reducedMotion: boolean;
  profile: MotionProfile;
}

export interface MonitorSample {
  cpu: number;
  memUsed: number;
  memTotal: number;
  diskUsed: number;
  diskTotal: number;
  netRx: number;
  netTx: number;
}

export interface AutoHideState {
  enabled: boolean;
  visible: boolean;
  pending: boolean;
}
