//! Read-only resource provenance: the frozen asset manifest and the release
//! boundary scanner.
//!
//! The forbidden pattern list lives in `assets/forbidden_patterns.json` rather
//! than in this module so the runtime source tree never spells out the legacy
//! and competitor names it must reject.

use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use sha2::{Digest, Sha256};

const MANIFEST_SCHEMA_VERSION: u32 = 1;

/// One inventoried input, mirrored from `assets/manifest.json`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct AssetRecord {
    pub path: String,
    pub source: String,
    pub author: String,
    pub license: String,
    pub sha256: String,
    #[serde(default)]
    pub width: Option<u32>,
    #[serde(default)]
    pub height: Option<u32>,
    pub allowed_use: String,
    pub verified_at: String,
}

#[derive(Debug, Deserialize)]
struct AssetManifest {
    schema_version: u32,
    assets: Vec<AssetRecord>,
}

#[derive(Debug, Deserialize)]
struct ForbiddenPatterns {
    forbidden_path_patterns: Vec<String>,
    forbidden_content_markers: Vec<String>,
    forbidden_extensions: Vec<String>,
    skipped_directory_names: Vec<String>,
    provenance_record_file_names: Vec<String>,
}

const FORBIDDEN_PATTERNS_JSON: &str = include_str!("../../assets/forbidden_patterns.json");

/// Absolute path of the frozen asset manifest, independent of the working
/// directory.
pub fn asset_manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join("manifest.json")
}

/// Load and validate the frozen asset manifest.
///
/// Panics when the manifest is missing, malformed, or uses an unknown schema
/// version: a broken provenance manifest must fail loudly.
pub fn load_asset_manifest() -> Vec<AssetRecord> {
    let path = asset_manifest_path();
    let source = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
    let manifest: AssetManifest = serde_json::from_str(&source)
        .unwrap_or_else(|error| panic!("cannot parse {}: {error}", path.display()));
    assert_eq!(
        manifest.schema_version,
        MANIFEST_SCHEMA_VERSION,
        "{} uses unsupported schema version {}",
        path.display(),
        manifest.schema_version
    );
    manifest.assets
}

/// Lowercase hex SHA-256 of a file's bytes.
pub fn sha256_file(path: impl AsRef<Path>) -> std::io::Result<String> {
    let bytes = fs::read(path)?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    Ok(format!("{:x}", hasher.finalize()))
}

/// Scan files and directories for release-boundary violations.
///
/// Forbidden path patterns are matched against each root's own relative path
/// so an unrelated ancestor directory name cannot flag a clean tree. Files are
/// also rejected for a forbidden extension (HTML/JS prototypes) or for file
/// text containing a forbidden content marker. Directories are searched
/// recursively; build and dependency directories are skipped. The crate's own
/// provenance records (`assets/manifest.json`, `assets/forbidden_patterns.json`)
/// are exempt from the content check because they must name the sources they
/// restrict. The result is sorted and deduplicated so CI output is stable.
pub fn scan_release(paths: &[PathBuf]) -> Vec<String> {
    let patterns = load_forbidden_patterns();
    let ancestor = common_ancestor(paths);
    let mut violations = Vec::new();
    for path in paths {
        let mut relative = ancestor
            .as_ref()
            .and_then(|ancestor| path.strip_prefix(ancestor).ok())
            .map(Path::to_path_buf)
            .unwrap_or_else(|| path.clone());
        if relative.as_os_str().is_empty() {
            relative = path.file_name().map(PathBuf::from).unwrap_or(relative);
        }
        scan_path(path, &relative, &patterns, &mut violations);
    }
    violations.sort();
    violations.dedup();
    violations
}

/// Deepest directory containing every scan input, so relative paths keep the
/// same shape whatever mix of roots and files the caller supplies.
fn common_ancestor(paths: &[PathBuf]) -> Option<PathBuf> {
    let mut paths = paths.iter();
    let mut ancestor: Vec<std::path::Component<'_>> = paths.next()?.components().collect();
    for path in paths {
        let components: Vec<_> = path.components().collect();
        let shared = ancestor
            .iter()
            .zip(&components)
            .take_while(|(left, right)| left == right)
            .count();
        ancestor.truncate(shared);
    }
    Some(ancestor.iter().collect())
}

fn load_forbidden_patterns() -> ForbiddenPatterns {
    serde_json::from_str(FORBIDDEN_PATTERNS_JSON)
        .expect("assets/forbidden_patterns.json must be valid")
}

fn scan_path(
    absolute: &Path,
    relative: &Path,
    patterns: &ForbiddenPatterns,
    violations: &mut Vec<String>,
) {
    if absolute.is_dir() {
        if is_skipped_directory(absolute, patterns) {
            return;
        }
        if let Some(violation) = path_violation(absolute, relative, patterns) {
            violations.push(violation);
            return;
        }
        let entries = match fs::read_dir(absolute) {
            Ok(entries) => entries,
            Err(error) => {
                violations.push(format!("{}: unreadable ({error})", display(absolute)));
                return;
            }
        };
        for entry in entries {
            match entry {
                Ok(entry) => {
                    let child = relative.join(entry.file_name());
                    scan_path(&entry.path(), &child, patterns, violations);
                }
                Err(error) => {
                    violations.push(format!("{}: unreadable ({error})", display(absolute)));
                }
            }
        }
    } else if absolute.is_file() {
        scan_file(absolute, relative, patterns, violations);
    } else {
        violations.push(format!("{}: missing or not a file", display(absolute)));
    }
}

fn is_skipped_directory(path: &Path, patterns: &ForbiddenPatterns) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| {
            patterns
                .skipped_directory_names
                .iter()
                .any(|skipped| skipped.eq_ignore_ascii_case(name))
        })
}

fn scan_file(
    absolute: &Path,
    relative: &Path,
    patterns: &ForbiddenPatterns,
    violations: &mut Vec<String>,
) {
    if let Some(violation) = path_violation(absolute, relative, patterns) {
        violations.push(violation);
        return;
    }
    if let Some(extension) = forbidden_extension(absolute, patterns) {
        violations.push(format!(
            "{}: forbidden file extension '.{extension}'",
            display(absolute)
        ));
        return;
    }
    if is_provenance_record(relative, patterns) {
        return;
    }
    let bytes = match fs::read(absolute) {
        Ok(bytes) => bytes,
        Err(error) => {
            violations.push(format!("{}: unreadable ({error})", display(absolute)));
            return;
        }
    };
    let text = String::from_utf8_lossy(&bytes).to_lowercase();
    for marker in &patterns.forbidden_content_markers {
        if text.contains(&marker.to_lowercase()) {
            violations.push(format!(
                "{}: forbidden content marker '{marker}'",
                display(absolute)
            ));
        }
    }
}

fn path_violation(
    absolute: &Path,
    relative: &Path,
    patterns: &ForbiddenPatterns,
) -> Option<String> {
    let normalized = display(relative).to_lowercase();
    patterns
        .forbidden_path_patterns
        .iter()
        .find(|pattern| contains_path_pattern(&normalized, pattern))
        .map(|pattern| format!("{}: forbidden path pattern '{pattern}'", display(absolute)))
}

fn forbidden_extension(path: &Path, patterns: &ForbiddenPatterns) -> Option<String> {
    let extension = path.extension()?.to_str()?.to_lowercase();
    patterns
        .forbidden_extensions
        .iter()
        .find(|forbidden| forbidden.eq_ignore_ascii_case(&extension))
        .cloned()
}

/// Only the manifest's own record files are exempt from the content check, and
/// only when they sit in an `assets/` directory.
fn is_provenance_record(relative: &Path, patterns: &ForbiddenPatterns) -> bool {
    let in_assets = relative
        .parent()
        .and_then(Path::file_name)
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.eq_ignore_ascii_case("assets"));
    in_assets
        && relative
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| {
                patterns
                    .provenance_record_file_names
                    .iter()
                    .any(|record| record.eq_ignore_ascii_case(name))
            })
}

/// Forward-slash form used both for matching and for messages.
fn display(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

/// Substring search that only matches on path-segment boundaries, so a
/// pattern never matches inside a longer directory name.
fn contains_path_pattern(normalized_path: &str, pattern: &str) -> bool {
    let pattern = pattern.to_lowercase();
    if pattern.is_empty() {
        return false;
    }
    let bytes = normalized_path.as_bytes();
    let mut offset = 0;
    while let Some(found) = normalized_path[offset..].find(&pattern) {
        let start = offset + found;
        let end = start + pattern.len();
        let starts_at_boundary = start == 0 || bytes[start - 1] == b'/';
        let ends_at_boundary = end == bytes.len() || bytes[end] == b'/';
        if starts_at_boundary && ends_at_boundary {
            return true;
        }
        offset = start + 1;
    }
    false
}
