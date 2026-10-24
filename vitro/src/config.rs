// config.rs — 配置与主题(Glassboard preferences.rs 思路的平铺版:分层默认 + 路径自愈 + 写前留档)
// NeXuS R11 持久化:~/.vitro/dock.toml(应用/插件/自动隐藏)+ theme.toml(五参数)
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct DockItem { pub name: String, pub path: String }

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct PluginSpec {
    pub name: String,
    pub code: String,
    #[serde(default)]
    pub icon: String, // 可选:SQ 图标键;缺省按名称猜测
}

#[derive(Serialize, Deserialize)]
pub(crate) struct DockConfig {
    pub items: Vec<DockItem>,
    #[serde(default)]
    pub plugins: Vec<PluginSpec>,
    #[serde(default)]
    pub autohide: bool,
}

impl Default for DockConfig {
    fn default() -> Self {
        Self {
            items: vec![
                DockItem { name: "Explorer".into(), path: "C:\\Windows\\explorer.exe".into() },
                DockItem { name: "浏览器".into(), path: "C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe".into() },
                DockItem { name: "记事本".into(), path: "C:\\Windows\\notepad.exe".into() },
                DockItem { name: "计算器".into(), path: "C:\\Windows\\System32\\calc.exe".into() },
                DockItem { name: "终端".into(), path: "C:\\Windows\\System32\\cmd.exe".into() },
            ],
            plugins: vec![
                PluginSpec { name: "时钟".into(), code: "api.set('12:34')".into(), icon: String::new() },
            ],
            autohide: false,
        }
    }
}

// 历史坑:TOML 双引号曾把 `\n` 转义成真换行写进路径,坏文件导致整份配置失效回退默认
// → 读:剥控制字符;解析失败把坏文件挪 .toml.corrupt 留证;写:先备份上一份到 .toml.bak
fn clean_path(p: &str) -> String {
    p.chars().filter(|c| !c.is_control()).collect::<String>().trim().to_string()
}

pub(crate) fn config_path() -> PathBuf {
    let home = std::env::var("USERPROFILE").unwrap_or_else(|_| "C:\\Users\\Public".into());
    PathBuf::from(&home).join(".vitro").join("dock.toml")
}

pub(crate) fn load_config() -> DockConfig {
    let p = config_path();
    if p.exists() {
        if let Ok(s) = std::fs::read_to_string(&p) {
            match toml::from_str::<DockConfig>(&s) {
                Ok(mut cfg) => {
                    let mut seen = std::collections::HashSet::new();
                    for it in &mut cfg.items { it.path = clean_path(&it.path); }
                    cfg.items.retain(|it| !it.path.is_empty() && seen.insert(it.path.to_lowercase()));
                    return cfg;
                }
                Err(e) => {
                    let bad = p.with_extension("toml.corrupt");
                    println!("[vitro] 配置解析失败({e})——已备份到 {} 并重建默认", bad.display());
                    let _ = std::fs::copy(&p, &bad);
                }
            }
        }
    }
    DockConfig::default()
}

pub(crate) fn save_config(cfg: &DockConfig) {
    let p = config_path();
    let _ = std::fs::create_dir_all(p.parent().unwrap());
    if p.exists() { let _ = std::fs::copy(&p, p.with_extension("toml.bak")); }
    if let Ok(s) = toml::to_string_pretty(cfg) { let _ = std::fs::write(&p, s); }
}

// ── 主题:取自 vitro-macos-parity.html 的 Tahoe 材质参数 ──
// rgba(0,0,0,.12) + blur(28px) saturate(1.12),accent #7a82cd
#[derive(Clone)]
pub(crate) struct Theme {
    pub alpha: f32,
    pub tint: [f32; 3],
    pub accent: [f32; 3],
    pub peak: f32,
    pub radius: f32,
}

impl Default for Theme {
    fn default() -> Self {
        Theme { alpha: 0.56, tint: [0.075, 0.085, 0.130], accent: [0.478, 0.510, 0.804], peak: 1.60, radius: 120.0 }
    }
}

#[derive(Serialize, Deserialize, Default)]
struct ThemeFile {
    alpha: f32,
    tint: [f32; 3],
    accent: [f32; 3],
    peak: f32,
    radius: f32,
}

pub(crate) fn theme_path() -> PathBuf {
    let home = std::env::var("USERPROFILE").unwrap_or_else(|_| "C:\\Users\\Public".into());
    PathBuf::from(&home).join(".vitro").join("theme.toml")
}

pub(crate) fn load_theme() -> Theme {
    let d = Theme::default();
    let p = theme_path();
    if let Ok(s) = std::fs::read_to_string(&p) {
        if let Ok(t) = toml::from_str::<ThemeFile>(&s) {
            let nz = |v: f32, dv: f32| if v == 0.0 { dv } else { v };
            return Theme {
                alpha: nz(t.alpha, d.alpha).clamp(0.05, 1.0),
                tint: if t.tint == [0.0; 3] { d.tint } else { t.tint },
                accent: if t.accent == [0.0; 3] { d.accent } else { t.accent },
                peak: nz(t.peak, d.peak).clamp(1.0, 2.0),
                radius: nz(t.radius, d.radius).clamp(40.0, 400.0),
            };
        }
    }
    d
}
