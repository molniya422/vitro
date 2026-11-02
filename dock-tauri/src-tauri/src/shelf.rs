//! Session-only file shelf. Pure path-list operations; no IO.
//!
//! The shelf intentionally does not persist: it is a scratch space for the
//! current session, like the macOS shelf.

use std::path::Path;

/// Upper bound on shelf entries. Small enough to stay a glanceable strip.
pub const SHELF_CAP: usize = 12;

fn same_path(a: &str, b: &str) -> bool {
    a.eq_ignore_ascii_case(b)
}

/// Add `path` unless it is empty, already present (case-insensitively), or the
/// shelf is full. Returns whether the list changed.
pub fn push_unique(items: &mut Vec<String>, path: &str) -> bool {
    let trimmed = path.trim();
    if trimmed.is_empty() || items.len() >= SHELF_CAP {
        return false;
    }
    if items.iter().any(|existing| same_path(existing, trimmed)) {
        return false;
    }
    items.push(trimmed.to_string());
    true
}

/// Remove `path` case-insensitively. Returns whether anything was removed.
pub fn remove(items: &mut Vec<String>, path: &str) -> bool {
    let before = items.len();
    items.retain(|existing| !same_path(existing, path));
    items.len() != before
}

/// Display name for a path: the final component, or the whole path for a root
/// such as `C:\` where `file_name` is `None`.
pub fn display_name(path: &str) -> String {
    Path::new(path)
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| path.to_string())
}
