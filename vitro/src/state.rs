// state.rs — 唯一可变状态 + 布局几何
// 坐标系约定:窗口是固定宽的透明画布,"玻璃药丸"按磁贴数自适应宽度居中绘制。
// 几何以药丸为原点;钩子/命中测试用屏幕坐标(窗口原点平移一次)。
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Direct2D::*;
use windows::Win32::Graphics::DirectWrite::IDWriteTextFormat;
use windows::Win32::UI::WindowsAndMessaging::HHOOK;

use crate::config::{PluginSpec, Theme};
use crate::core::geometry;
use crate::core::model::TileId;
use crate::gpu::Gpu;
use std::collections::HashMap;

pub(crate) const WIN_W: i32 = 1024;   // 画布固定宽(≤屏宽-160 时自动缩小,启动时定死)
pub(crate) const WIN_H: i32 = 320;    // 220 子码头缓冲 + 68 面板 + 32 倒影
pub(crate) const PANEL_TOP: f32 = 220.0;
pub(crate) const PANEL_H: f32 = 68.0;
pub(crate) const ICON: f32 = 48.0;
pub(crate) const GAP: f32 = 8.0;
pub(crate) const PAD: f32 = 16.0;     // 药丸左右内边距
pub(crate) const BASE_UP: f32 = 10.0; // 图标底距面板底
pub(crate) const WM_APP_MOUSE: u32 = 0x8000;
pub(crate) const WM_APP_TRAY: u32 = 0x8010;
pub(crate) const TIMER_DWELL: u32 = 2;
pub(crate) const TIMER_ANIM: u32 = 3;
pub(crate) const TIMER_HIDE: u32 = 4;

pub(crate) struct Tile {
    pub id: TileId,
    pub name: String,
    pub path: String,
    pub bitmap: Option<ID2D1Bitmap1>,   // Shell 提取的真实图标
    pub fallback: Option<ID2D1Bitmap1>, // 无图标时的字符位图
    pub is_plugin: bool,
    pub is_folder: bool,
    pub color_idx: usize,
    pub sq: Option<&'static str>,       // SQ 图标键(插件/兜底)
    pub label: Option<String>,          // 插件徽章文本(api.set 值)
}

pub(crate) struct SubdockItem {
    pub name: String,
    pub path: String,
    pub bitmap: Option<ID2D1Bitmap1>,
    pub fallback: Option<ID2D1Bitmap1>,
    pub sq: Option<&'static str>,
    pub color_idx: usize,
}

pub(crate) struct Subdock {
    pub parent: TileId,
    pub items: Vec<SubdockItem>,
    pub width: i32,
    pub left: f32,
    pub top: f32,
    pub progress: f32,
    pub hovered: Option<usize>,
    pub closing: bool,
}

pub(crate) struct State {
    pub hwnd: HWND,
    pub tip_hwnd: HWND,
    pub gpu: Gpu,
    pub tiles: Vec<Tile>,
    pub subdock: Option<Subdock>,
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
    pub plugin_fmt: IDWriteTextFormat,
    pub plugin_fmt_sm: IDWriteTextFormat,
    pub palette: Vec<ID2D1SolidColorBrush>,
    pub sq: HashMap<&'static str, ID2D1Bitmap1>,
    pub grad_refl: ID2D1Brush,
    pub grad_lin: ID2D1LinearGradientBrush,
    pub layer: ID2D1Layer,
    pub noise: Option<ID2D1Brush>, // Sonoma 四层材质·颗粒纹理层(0.3% 均匀噪点,消塑料感/抗色带)
    // 输入
    pub hook: HHOOK,
    // 几何
    pub win_w: i32,
    pub win_x: i32,
    pub win_y: i32,     // 滑动动画改写
    pub base_y: i32,
    pub screen_h: i32,
    pub pill_w: i32,
    // 召唤/隐藏(NeXuS R9)
    pub slide: f32,
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

    pub(crate) fn subdock_hit_test(&self, sx: i32, sy: i32) -> Option<usize> {
        let sub = self.subdock.as_ref()?;
        if sub.closing || sub.progress < 0.999 {
            return None;
        }
        let x = sx as f32 - self.win_x as f32;
        let y = sy as f32 - self.win_y as f32;
        if x < sub.left || x > sub.left + sub.width as f32 || y < sub.top || y > sub.top + PANEL_H { return None; }
        let size = tile_size(sub.width, sub.items.len());
        for i in 0..sub.items.len() {
            let cx = sub.left + PAD + size / 2.0 + i as f32 * (size + GAP);
            let cy = sub.top + PANEL_H - BASE_UP - size / 2.0;
            if (x - cx).abs() <= size * 0.55 && (y - cy).abs() <= size * 0.55 { return Some(i); }
        }
        None
    }

    pub(crate) fn subdock_contains(&self, sx: i32, sy: i32) -> bool {
        let Some(sub) = self.subdock.as_ref() else { return false; };
        let x = sx as f32 - self.win_x as f32;
        let y = sy as f32 - self.win_y as f32;
        x >= sub.left && x <= sub.left + sub.width as f32 && y >= sub.top - 16.0 && y <= sub.top + PANEL_H
    }

    pub(crate) fn move_tile(&mut self, from: usize, to: usize) -> bool {
        let n = self.tiles.len();
        if !crate::core::model::move_item(&mut self.tiles, from, to) { return false; }
        if self.scale_anim.len() >= n { crate::core::model::move_item(&mut self.scale_anim, from, to); }
        if self.running.len() >= n { crate::core::model::move_item(&mut self.running, from, to); }
        crate::core::model::move_item(&mut self.cpu_prev, from, to);
        crate::core::model::move_item(&mut self.cpu_pct, from, to);
        self.relayout();
        true
    }

    pub(crate) fn pill_x(&self) -> f32 { (self.win_w as f32 - self.pill_w as f32) / 2.0 }

    // 磁贴中心(窗口局部):x 沿药丸,y 底对齐
    pub(crate) fn tile_center_local(&self, i: usize) -> (f32, f32, f32) {
        let (x, y, sz) = geometry::tile_center_local(
            self.pill_w, self.tiles.len(), i, ICON, GAP, PAD, PANEL_H, BASE_UP,
        );
        (x + self.pill_x(), PANEL_TOP + y, sz)
    }

    // 磁贴中心(屏幕坐标,钩子/命中用)
    pub(crate) fn tile_center_screen(&self, i: usize) -> (f32, f32, f32) {
        let (x, y, sz) = self.tile_center_local(i);
        (self.win_x as f32 + x, self.win_y as f32 + y, sz)
    }

    pub(crate) fn pill_rect_screen(&self) -> RECT {
        let l = self.win_x + self.pill_x() as i32;
        let top = self.win_y + PANEL_TOP as i32;
        RECT { left: l, top, right: l + self.pill_w, bottom: top + PANEL_H as i32 }
    }

    pub(crate) fn hit_test(&self, sx: i32, sy: i32) -> Option<usize> {
        for i in 0..self.tiles.len() {
            let (cx, _, base_size) = self.tile_center_screen(i);
            let scale = self.scale_anim.get(i).copied().unwrap_or(1.0);
            let size = base_size * scale;
            let cy = self.win_y as f32 + PANEL_TOP + PANEL_H - BASE_UP - size / 2.0;
            if (sx as f32 - cx).abs() <= size * 0.55 && (sy as f32 - cy).abs() <= size * 0.55 { return Some(i); }
        }
        None
    }

    pub(crate) fn cursor_inside_pill(&self, cursor: (f32, f32)) -> bool {
        let r = self.pill_rect_screen();
        cursor.0 >= r.left as f32 && cursor.0 <= r.right as f32
            && cursor.1 >= r.top as f32 && cursor.1 <= r.bottom as f32
    }
}

pub(crate) fn pill_w_for(n: usize, win_w: i32) -> i32 {
    geometry::pill_width(n, win_w, ICON, GAP, PAD)
}

pub(crate) fn tile_size(pill_w: i32, n: usize) -> f32 {
    geometry::tile_size(pill_w, n, ICON, GAP, PAD)
}
