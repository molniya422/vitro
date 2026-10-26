//! Shell icon extraction and a disk cache served to the UI as data URLs.

use base64::Engine;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

pub struct IconCache {
    dir: PathBuf,
}

impl IconCache {
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    /// Cache key: sha256 of the lowercased, forward-slashed path plus icon size.
    pub fn key(path: &str, size: u32) -> String {
        let normalized = path.replace('\\', "/").to_ascii_lowercase();
        let mut hasher = Sha256::new();
        hasher.update(normalized.as_bytes());
        hasher.update(b"@");
        hasher.update(size.to_string().as_bytes());
        format!("{:x}", hasher.finalize())
    }

    fn file(&self, key: &str) -> PathBuf {
        self.dir.join(format!("{key}.png"))
    }

    pub fn store(&self, key: &str, png: &[u8]) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.dir)?;
        std::fs::write(self.file(key), png)
    }

    /// Return a data URL, or `None` on a miss or an unreadable entry.
    pub fn read(&self, key: &str) -> Option<String> {
        let bytes = std::fs::read(self.file(key)).ok()?;
        if bytes.is_empty() {
            return None;
        }
        let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
        Some(format!("data:image/png;base64,{encoded}"))
    }
}

/// Data URLs for every app, extracting and caching on first use.
pub fn icons_for(apps: &[crate::apps::AppEntry], size: u32) -> Vec<Option<String>> {
    let dir = std::env::var("APPDATA")
        .map(|base| PathBuf::from(base).join("DockApp/icons"))
        .unwrap_or_else(|_| std::env::temp_dir().join("dockapp-icons"));
    let cache = IconCache::new(dir);
    apps.iter()
        .map(|app| {
            let key = IconCache::key(&app.id, size);
            if let Some(url) = cache.read(&key) {
                return Some(url);
            }
            let png = extract_png(Path::new(&app.id), size)?;
            cache.store(&key, &png).ok()?;
            cache.read(&key)
        })
        .collect()
}

/// Extract an icon for `path` as a PNG sized to `size` pixels.
///
/// Prefers the 256 px jumbo image list, then 48 px, then the 16 px system icon.
/// Icons smaller than `size` are returned at their native size rather than
/// upscaled, so a degraded 16 px fallback stays visible instead of masquerading
/// as a large icon.
#[cfg(windows)]
pub fn extract_png(path: &Path, size: u32) -> Option<Vec<u8>> {
    extract::extract_icon(path, size)
}

#[cfg(not(windows))]
pub fn extract_png(_path: &Path, _size: u32) -> Option<Vec<u8>> {
    None
}

#[cfg(windows)]
mod extract {
    use std::mem::size_of;
    use std::os::windows::ffi::OsStrExt;

    use windows::core::PCWSTR;
    use windows::Win32::Graphics::Gdi::{
        DeleteObject, GetDC, GetDIBits, GetObjectW, ReleaseDC, BITMAP, BITMAPINFO,
        BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, HBITMAP, HDC, HGDIOBJ,
    };
    use windows::Win32::Storage::FileSystem::FILE_ATTRIBUTE_NORMAL;
    use windows::Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED};
    use windows::Win32::UI::Controls::{IImageList, ILD_TRANSPARENT};
    use windows::Win32::UI::Shell::{
        SHGetFileInfoW, SHGetImageList, SHFILEINFOW, SHGFI_FLAGS, SHGFI_ICON, SHGFI_SYSICONINDEX,
        SHIL_EXTRALARGE, SHIL_JUMBO,
    };
    use windows::Win32::UI::WindowsAndMessaging::{DestroyIcon, GetIconInfo, HICON, ICONINFO};

    use std::path::Path;

    fn wide(path: &Path) -> Vec<u16> {
        // Start Menu entries are stored with forward slashes; the Shell's path
        // parser wants native separators, as `apps::launch` already does.
        let native = path.to_string_lossy().replace('/', "\\");
        std::ffi::OsStr::new(&native)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect()
    }

    fn file_info(path: &Path, flags: SHGFI_FLAGS) -> Option<SHFILEINFOW> {
        let name = wide(path);
        let mut info = SHFILEINFOW::default();
        // SAFETY: `name` is NUL-terminated and outlives the call; `info` is a
        // local out-parameter of the declared size.
        let ok = unsafe {
            SHGetFileInfoW(
                PCWSTR(name.as_ptr()),
                FILE_ATTRIBUTE_NORMAL,
                Some(&mut info),
                size_of::<SHFILEINFOW>() as u32,
                flags,
            )
        };
        (ok != 0).then_some(info)
    }

    /// Best available HICON: jumbo (256 px) -> extra large (48 px) -> small.
    fn best_icon(path: &Path) -> Option<HICON> {
        // The system image-list index is what lets us borrow the large Shell
        // image lists; `SHGFI_ICON` alone would always be 16 px.
        if let Some(info) = file_info(path, SHGFI_SYSICONINDEX) {
            for shil in [SHIL_JUMBO, SHIL_EXTRALARGE] {
                // SAFETY: SHGetImageList returns a Shell-owned borrowed list.
                let Ok(list) = (unsafe { SHGetImageList::<IImageList>(shil as i32) }) else {
                    continue;
                };
                // SAFETY: the index came from the Shell for this path and the
                // returned HICON is ours to destroy.
                if let Ok(icon) = unsafe { list.GetIcon(info.iIcon, ILD_TRANSPARENT.0) } {
                    if !icon.is_invalid() {
                        return Some(icon);
                    }
                }
            }
        }
        // Last resort: the small system icon.
        file_info(path, SHGFI_ICON)
            .map(|info| info.hIcon)
            .filter(|icon| !icon.is_invalid())
    }

    fn native_size(bitmap: HBITMAP) -> Option<(u32, u32, u16)> {
        let mut info = BITMAP::default();
        // SAFETY: `info` is a valid BITMAP out-parameter of the declared size.
        let copied = unsafe {
            GetObjectW(
                HGDIOBJ(bitmap.0),
                size_of::<BITMAP>() as i32,
                Some(&mut info as *mut _ as *mut _),
            )
        };
        if copied == 0 || info.bmWidth <= 0 || info.bmHeight <= 0 {
            return None;
        }
        Some((info.bmWidth as u32, info.bmHeight as u32, info.bmBitsPixel))
    }

    /// Read a bitmap into a top-down DIB of the requested bit depth.
    fn dib(dc: HDC, bitmap: HBITMAP, width: u32, height: u32, bit_count: u16) -> Option<Vec<u8>> {
        let header = BITMAPINFOHEADER {
            biSize: size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: width as i32,
            biHeight: -(height as i32), // negative = top-down rows
            biPlanes: 1,
            biBitCount: bit_count,
            biCompression: BI_RGB.0,
            ..Default::default()
        };
        let mut bitmap_info = BITMAPINFO {
            bmiHeader: header,
            ..Default::default()
        };
        let stride = ((width as usize * bit_count as usize).div_ceil(32)) * 4;
        let mut buffer = vec![0u8; stride * height as usize];
        // SAFETY: the DIB description matches the bitmap's own dimensions and
        // `buffer` holds `height` rows at `bit_count` bits per pixel.
        let lines = unsafe {
            GetDIBits(
                dc,
                bitmap,
                0,
                height,
                Some(buffer.as_mut_ptr() as *mut _),
                &mut bitmap_info,
                DIB_RGB_COLORS,
            )
        };
        (lines == height as i32).then_some(buffer)
    }

    /// Apply a monochrome AND mask: set bits are transparent.
    fn apply_mask(rgba: &mut [u8], mask: &[u8], width: u32, height: u32) {
        let stride = (width as usize).div_ceil(32) * 4;
        for y in 0..height as usize {
            for x in 0..width as usize {
                let transparent = (mask[y * stride + x / 8] >> (7 - (x % 8))) & 1 != 0;
                let alpha = if transparent { 0 } else { 255 };
                rgba[(y * width as usize + x) * 4 + 3] = alpha;
            }
        }
    }

    fn hicon_to_png(icon: HICON, size: u32) -> Option<Vec<u8>> {
        // SAFETY: every GDI object and device context created here is released
        // before returning; the icon itself is owned by the caller.
        unsafe {
            let mut icon_info = ICONINFO::default();
            if GetIconInfo(icon, &mut icon_info).is_err() {
                return None;
            }
            let (color, mask) = (icon_info.hbmColor, icon_info.hbmMask);

            let result = (|| -> Option<Vec<u8>> {
                // The bitmap's own dimensions, not the requested size: GetDIBits
                // copies at native resolution and does not scale.
                let (width, height, bits) = native_size(color)?;
                let dc = GetDC(None);
                if dc.is_invalid() {
                    return None;
                }

                let read = (|| -> Option<Vec<u8>> {
                    let mut rgba = dib(dc, color, width, height, 32)?;

                    // 32 bpp colour bitmaps normally carry their own alpha. When
                    // they do not, use the AND mask; without a mask, the icon is
                    // fully opaque.
                    let has_alpha = bits >= 32 && rgba.as_chunks::<4>().0.iter().any(|p| p[3] != 0);
                    if !has_alpha {
                        let mask_bits = if mask.is_invalid() {
                            None
                        } else {
                            dib(dc, mask, width, height, 1)
                        };
                        match mask_bits {
                            Some(mask_bits) => apply_mask(&mut rgba, &mask_bits, width, height),
                            None => {
                                for pixel in rgba.as_chunks_mut::<4>().0 {
                                    pixel[3] = 255;
                                }
                            }
                        }
                    }

                    // Shell icons are BGRA; `image` expects RGBA.
                    for pixel in rgba.as_chunks_mut::<4>().0 {
                        pixel.swap(0, 2);
                    }

                    let rgba = image::RgbaImage::from_raw(width, height, rgba)?;
                    let edge = size.min(width.max(height));
                    let scaled = if width == edge && height == edge {
                        rgba
                    } else {
                        image::imageops::resize(
                            &rgba,
                            edge,
                            edge,
                            image::imageops::FilterType::Lanczos3,
                        )
                    };
                    let mut png = std::io::Cursor::new(Vec::new());
                    scaled.write_to(&mut png, image::ImageFormat::Png).ok()?;
                    Some(png.into_inner())
                })();

                ReleaseDC(None, dc);
                read
            })();

            if !color.is_invalid() {
                let _ = DeleteObject(HGDIOBJ(color.0));
            }
            if !mask.is_invalid() {
                let _ = DeleteObject(HGDIOBJ(mask.0));
            }
            result
        }
    }

    pub fn extract_icon(path: &Path, size: u32) -> Option<Vec<u8>> {
        if size == 0 {
            return None;
        }
        // SAFETY: COM is initialised for the duration of this call and
        // uninitialised only when this call initialised it, mirroring
        // `apps::resolve_shortcut_target`.
        unsafe {
            let initialised = CoInitializeEx(None, COINIT_APARTMENTTHREADED).is_ok();
            let result = best_icon(path).and_then(|icon| {
                let png = hicon_to_png(icon, size);
                // SAFETY: the handle is no longer used after this point.
                let _ = DestroyIcon(icon);
                png
            });
            if initialised {
                CoUninitialize();
            }
            result
        }
    }
}
