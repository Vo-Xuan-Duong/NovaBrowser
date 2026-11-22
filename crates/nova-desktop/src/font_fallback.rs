//! OS font fallback for the embedded Obscura renderer.
//!
//! The renderer intentionally ships deterministic Latin fonts. On Windows,
//! these do not cover Han, Hangul, Hiragana, Katakana or many other scripts.
//! Load a small, explicit set of installed faces before first rendering
//! instead of scanning the entire Windows Fonts directory.
//!
//! System font data stays on this machine: no downloads or bundled Microsoft
//! font redistribution. Missing optional faces simply remain unavailable.

#[cfg(any(test, windows))]
use std::path::{Path, PathBuf};

// Priority is deliberate: Unicode CJK fonts before niche script fallbacks.
// Font loading itself enforces per-file and aggregate byte limits.
#[cfg(any(test, windows))]
const WINDOWS_FALLBACK_FONTS: &[&str] = &[
    "msyh.ttc",      // Microsoft YaHei: Simplified Chinese
    "msyhbd.ttc",
    "msyhl.ttc",    // additional YaHei face when installed
    "msjh.ttc",      // Microsoft JhengHei: Traditional Chinese
    "simsun.ttc",    // SimSun / NSimSun
    "mingliu.ttc",   // MingLiU
    "meiryo.ttc",    // Japanese
    "yugothm.ttc",   // Yu Gothic
    "malgun.ttf",    // Korean
    "leelawui.ttf",  // Thai
    "nirmala.ttf",   // Nirmala UI: South Asian scripts
    "mangal.ttf",    // Devanagari
    "segoeui.ttf",   // Windows common fallback
    "seguisym.ttf",  // Unicode symbol fallback
];

/// Return only installed, real files, preserving the script priority above.
#[cfg(any(test, windows))]
fn select_installed_fonts(directory: &Path) -> Vec<PathBuf> {
    WINDOWS_FALLBACK_FONTS.iter()
        .map(|filename| directory.join(filename))
        .filter(|path| path.is_file())
        .collect()
}

#[cfg(windows)]
fn system_fonts_directory() -> Option<PathBuf> {
    let windows = std::env::var_os("WINDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\Windows"));
    let directory = windows.join("Fonts");
    directory.is_dir().then_some(directory)
}

#[cfg(windows)]
pub fn configure_desktop_fonts() {
    let Some(directory) = system_fonts_directory() else {
        eprintln!("NovaBrowser: Windows Fonts directory was not found; using bundled fonts");
        return;
    };
    let paths = select_installed_fonts(&directory);
    if paths.is_empty() {
        eprintln!("NovaBrowser: no optional Unicode fallback fonts found");
        return;
    }
    if !obscura_render::configure_font_files(paths) {
        eprintln!("NovaBrowser: font database was already initialized before fallback setup");
    }
}

#[cfg(not(windows))]
pub fn configure_desktop_fonts() {
    // Linux/macOS support can use configure_font_files with distro-specific
    // fallback locations later. Do not change renderer behavior silently.
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unavailable_fonts_are_skipped_and_priority_is_preserved() {
        let suffix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let root = std::env::temp_dir().join(format!("nova-font-probe-{}-{suffix}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        for name in ["malgun.ttf", "msyh.ttc", "unlisted.ttf"] {
            std::fs::write(root.join(name), b"not a real font").unwrap();
        }
        let chosen = select_installed_fonts(&root);
        let names: Vec<_> = chosen.iter().map(|path| path.file_name().unwrap().to_str().unwrap()).collect();
        assert_eq!(names, vec!["msyh.ttc", "malgun.ttf"]);
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn missing_fonts_directory_is_safe() {
        let root = std::env::temp_dir().join(format!(
            "nova-fonts-missing-{}", std::process::id()
        ));
        assert!(select_installed_fonts(&root.join("intentionally-absent")).is_empty());
    }
}
