/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::fs;

// Third Party Imports
use dokime_runtime::brand::theme::{self, Error};
use tempfile::tempdir;

// Local Imports

/* === Constants === */
const TYPOGRAPHY: &str = ":root { --brand-font-sans: sample; --brand-font-heading: sample; --brand-font-mono: sample; --brand-text-xs: sample; --brand-text-xl: sample; --brand-text-2xl: sample; --brand-text-3xl: sample; --brand-text-4xl: sample; --brand-text-5xl: sample; --brand-text-6xl: sample; --brand-text-7xl: sample; }";
const PALETTE: &str = "--background: #123456; --foreground: #123456; --card: #123456; --card-foreground: #123456; --popover: #123456; --popover-foreground: #123456; --primary: #123456; --primary-foreground: #123456; --secondary: #123456; --secondary-foreground: #123456; --muted: #123456; --muted-foreground: #123456; --accent: #123456; --accent-foreground: #123456; --destructive: #123456; --success: #123456; --success-foreground: #123456; --warning: #123456; --warning-foreground: #123456; --info: #123456; --info-foreground: #123456; --border: #123456; --input: #123456; --ring: #123456; --chart-1: #abcdef; --chart-2: #abcdef; --chart-3: #abcdef; --chart-4: #abcdef; --chart-5: #abcdef; --chart-6: #abcdef; --chart-7: #abcdef; --chart-8: #abcdef;";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */
fn load_typography(dir: &std::path::Path, css: &str) -> Result<String, Error> {
    fs::create_dir_all(dir.join("theme")).unwrap();
    fs::write(dir.join("theme/typography.css"), format!("{}\n{css}", TYPOGRAPHY)).unwrap();
    fs::write(dir.join("theme/base.css"), "").unwrap();
    fs::write(dir.join("theme/light.css"), format!(":root {{ {PALETTE} }}")).unwrap();
    fs::write(dir.join("theme/dark.css"), format!(".dark {{ {PALETTE} }}")).unwrap();

    theme::load(dir).map(|theme| theme.typography_css)
}

/* === Tests === */

#[test]
fn local_woff2_uris() {
    let dir = tempdir().unwrap();
    fs::create_dir(dir.path().join("fonts")).unwrap();
    fs::write(dir.path().join("fonts/workshop.woff2"), b"woff2-fixture").unwrap();

    let css = load_typography(dir.path(), "@font-face { src: url(\"fonts/workshop.woff2\") format(\"woff2\"); }").unwrap();

    assert!(css.contains("url(\"data:font/woff2;base64,d29mZjItZml4dHVyZQ==\")"));
    assert!(!css.contains("fonts/workshop.woff2"));
}

#[test]
fn missing_local_refused() {
    let dir = tempdir().unwrap();
    let error = load_typography(dir.path(), "a { src: url(fonts/missing.woff2); }").unwrap_err();
    assert!(matches!(error, Error::FontRead { .. }));
}

#[test]
fn parent_directory_refused() {
    let dir = tempdir().unwrap();
    let error = load_typography(dir.path(), "a { src: url(../outside.woff2); }").unwrap_err();
    assert!(matches!(error, Error::UnsafeUrl { .. }));
}

#[test]
fn remote_absolute_refused() {
    let dir = tempdir().unwrap();
    for url in ["https://example.com/font.woff2", "/tmp/font.woff2"] {
        let css = format!("a {{ src: url(\"{url}\"); }}");
        let error = load_typography(dir.path(), &css).unwrap_err();
        assert!(matches!(error, Error::UnsafeUrl { .. }));
    }
}

#[test]
fn unsupported_font_refused() {
    let dir = tempdir().unwrap();
    let error = load_typography(dir.path(), "a { src: url(fonts/font.svg); }").unwrap_err();
    assert!(matches!(error, Error::UnsupportedFont { .. }));
}

#[cfg(unix)]
#[test]
fn symlink_font_refused() {
    use std::os::unix::fs::symlink;

    let brand = tempdir().unwrap();
    let outside = tempdir().unwrap();
    fs::write(outside.path().join("font.woff2"), b"outside").unwrap();
    fs::create_dir(brand.path().join("fonts")).unwrap();
    symlink(outside.path().join("font.woff2"), brand.path().join("fonts/escaped.woff2")).unwrap();

    let error = load_typography(brand.path(), "a { src: url(fonts/escaped.woff2); }").unwrap_err();
    assert!(matches!(error, Error::UnsafeUrl { .. }));
}

#[test]
fn compiled_bundle_filesystem() {
    let authored = theme::load_from(&theme::Bundled).unwrap();

    assert!(authored.typography_css.contains("data:font/woff2;base64,"));
    assert!(theme::Bundled::manifest().contains("[app]"));
    assert!(theme::Bundled::file("logos/contained-dark.svg").is_some());
}
