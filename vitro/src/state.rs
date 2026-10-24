// state.rs — 唯一可变状态 + 布局几何
// 坐标系约定:窗口是固定宽的透明画布,"玻璃药丸"按磁贴数自适应宽度居中绘制。
// 几何以药丸为原点;钩子/命中测试用屏幕坐标(窗口原点平移一次)。
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Direct2D::*;
use windows::Win32::UI::WindowsAndMessaging::HHOOK;

use crate::config::{PluginSpec, Theme};
use crate::gpu::Gpu;
use std::collections::HashMap;

pub(crate) const WIN_W: i32 = 1024;   // 画布固定宽(≤屏宽-160 时自动缩小,启动时定死)
pub(crate) const WIN_H: i32 = 124;    // 92 面板 + 32 倒影
pub(crate) const PANEL_H: f32 = 92.0;
pub(crate) const ICON: f32 = 56.0;
pub(crate) const GAP: f32 = 13.0;
pub(crate) const PAD: f32 = 24.0;     // 药丸左右内边距
pub(crate) const BASE_UP: f32 = 12.0; // 图标底距面板底
pub(crate) const WM_APP_MOUSE: u32 = 0x8000;
pub(crate) const WM_APP_TRAY: u32 = 0x8010;
pub(crate) const TIMER_DWELL: u32 = 2;
pub(crate) const TIMER_ANIM: u32 = 3;
pub(crate) const TIMER_HIDE: u32 = 4;

pub(crate) struct Tile {
    pub name: String,
    pub path: String,
    pub bitmap: Option<ID2D1Bitmap1>,   // Shell 提取的真实图标
    pub fallback: Option<ID2D1Bitmap1>, // 无图标时的字符位图
    pub is_plugin: bool,
    pub color_idx: usize,
    pub sq: Option<&'static str>,       // SQ 图标键(插件/兜底)
    pub label: Option<String>,          // 插件徽章文本(api.set 值)
}

pub(crate) struct State {
    pub hwnd: HWND,
    pub tip_hwnd: HWND,
    pub gpu: Gpu,
    pub tiles: Vec<Tile>,
    // 画笔(主题相关,Ctrl+Alt+R 重建)
    pub glass: ID2D1SolidColorBrush,
    pub white: ID2D1SolidColorBrush,
    pub accent: ID2D1SolidColorBrush,
    pub halo: ID2D1SolidColorBrush,
    pub hi: ID2D1SolidColorBrush,
    pub lo: ID2D1SolidColorBrush,
    pub border: ID2D1SolidColorBrush,
    pub soft: ID2D1SolidColorBrush,
    pub pill: ID2D1SolidColorBrush,
    pub palette: Vec<ID2D1SolidColorBrush>,
    pub sq: HashMap<&'static str, ID2D1Bitmap1>,
    pub grad_refl: ID2D1Brush,
    pub grad_lin: ID2D1LinearGradientBrush,
    pub layer: ID2D1Layer,
    pub noise: Option<ID2D1Brush>, // Sonoma 四层材质·颗粒纹理层(0.3% 均匀噪点,消塑料感/抗色带)
    // 输入
    pub cursor: (f32, f32), // 屏幕坐标
    pub hover_idx: Option<usize>,
    pub hover_since: u32,
    pub tip_shown_for: Option<usize>,
    pub hook: HHOOK,
    pub was_near_flag: bool,
    pub last_post_tick: u32,
    // 几何
    pub win_w: i32,
    pub win_x: i32,
    pub win_y: i32,     // 滑动动画改写
    pub base_y: i32,
    pub screen_h: i32,
    pub pill_w: i32,
    // 召唤/隐藏(NeXuS R9)
    pub slide: f32,
    pub slide_target: f32,
    pub hide_armed: bool,
    pub autohide: bool,
    // 动效/呼吸
    pub low_power: bool,
    pub anim_on: bool,
    pub cpu_prev: Vec<(u64, u64)>,
    pub cpu_pct: Vec<f32>,
    pub last_anim_tick: u32,
    pub backdrop_ok: bool,
    pub scale_anim: Vec<f32>,
    pub running: Vec<Option<HANDLE>>,
    // 配置
    pub plugins: Vec<PluginSpec>,
    pub theme: Theme,
}

pub(crate) static mut STATE: Option<State> = None;
pub(crate) unsafe fn s() -> &'static mut State { STATE.as_mut().unwrap() }

impl State {
    // 磁贴数变化后调用:重算药丸宽 + 对齐各动态数组
    pub(crate) fn relayout(&mut self) {
        let n = self.tiles.len();
        self.pill_w = pill_w_for(n, self.win_w);
        let cap = n.max(1);
        if self.scale_anim.len() != cap { self.scale_anim = vec![1.0; cap]; }
        if self.running.len() < cap { self.running.resize(cap, None); }
        if self.cpu_prev.len() != cap { self.cpu_prev = vec![(0, 0); cap]; self.cpu_pct = vec![0.0; cap]; }
    }

    pub(crate) fn pill_x(&self) -> f32 { (self.win_w as f32 - self.pill_w as f32) / 2.0 }

    // 磁贴中心(窗口局部):x 沿药丸,y 底对齐
    pub(crate) fn tile_center_local(&self, i: usize) -> (f32, f32, f32) {
        let sz = tile_size(self.pill_w, self.tiles.len());
        let x = self.pill_x() + PAD + sz / 2.0 + i as f32 * (sz + GAP);
        let y = PANEL_H - BASE_UP - sz / 2.0;
        (x, y, sz)
    }

    // 磁贴中心(屏幕坐标,钩子/命中用)
    pub(crate) fn tile_center_screen(&self, i: usize) -> (f32, f32, f32) {
        let (x, y, sz) = self.tile_center_local(i);
        (self.win_x as f32 + x, self.win_y as f32 + y, sz)
    }

    pub(crate) fn pill_rect_screen(&self) -> RECT {
        let l = self.win_x + self.pill_x() as i32;
        RECT { left: l, top: self.win_y, right: l + self.pill_w, bottom: self.win_y + PANEL_H as i32 }
    }

    pub(crate) fn hit_test(&self, sx: i32, sy: i32) -> Option<usize> {
        for i in 0..self.tiles.len() {
            let (cx, cy, sz) = self.tile_center_screen(i);
            if (sx as f32 - cx).abs() <= sz * 0.75 && (sy as f32 - cy).abs() <= sz * 0.75 { return Some(i); }
        }
        None
    }

    pub(crate) fn cursor_inside_pill(&self) -> bool {
        let r = self.pill_rect_screen();
        self.cursor.0 >= r.left as f32 && self.cursor.0 <= r.right as f32
            && self.cursor.1 >= r.top as f32 && self.cursor.1 <= r.bottom as f32
    }
}

pub(crate) fn pill_w_for(n: usize, win_w: i32) -> i32 {
    if n == 0 { return (2.0 * PAD) as i32; }
    let w = n as f32 * ICON + (n - 1) as f32 * GAP + 2.0 * PAD;
    (w as i32).min(win_w - 40).max(2 * PAD as i32)
}

pub(crate) fn tile_size(pill_w: i32, n: usize) -> f32 {
    if n == 0 { return ICON; }
    let avail = (pill_w as f32 - 2.0 * PAD).max(60.0);
    ((avail / n as f32) - GAP).clamp(34.0, ICON)
}
