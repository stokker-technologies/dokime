/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::{collections::BTreeMap, path::Path};

use dokime_organization::{
    Materialized,
    theme::{self, Source},
};

fn repository() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn compiled_defaults_keep_the_existing_asset_bytes() {
    let root = repository();
    let directory = theme::Directory::new(&root.join("organization")).unwrap();
    let text = std::fs::read_to_string(root.join("organization/organization.toml")).unwrap();
    let pack = Materialized::read(&text, &directory).unwrap();
    let generated = root.join("crates/dokime-gui/src/generated/organization");
    assert_eq!(pack.default_json().unwrap(), std::fs::read_to_string(generated.join("default-organization.json")).unwrap());
    assert_eq!(pack.stylesheet().unwrap(), std::fs::read_to_string(generated.join("theme.css")).unwrap());
}

struct Overlay {
    directory: theme::Directory,
    files: BTreeMap<String, Vec<u8>>,
}

impl Source for Overlay {
    fn read(&self, relative: &str) -> std::io::Result<Vec<u8>> {
        match self.files.get(relative) {
            Some(bytes) => Ok(bytes.clone()),
            None => self.directory.read(relative),
        }
    }
}

#[test]
fn white_label_assets_share_one_materialized_presentation() {
    let directory = repository().join("organization");
    let mut manifest: toml::Value = toml::from_str(&std::fs::read_to_string(directory.join("organization.toml")).unwrap()).unwrap();
    manifest["app"]["label"] = "Bench Company".into();
    manifest["app"]["title_template"] = "Lab — {app}".into();
    let pack = Materialized::read(&toml::to_string(&manifest).unwrap(), &theme::Directory::new(&directory).unwrap()).unwrap();
    let json: serde_json::Value = serde_json::from_str(&pack.default_json().unwrap()).unwrap();
    assert_eq!(json["label"], "Bench Company");
    assert_eq!(json["title"], "Lab — Bench Company");
    assert_eq!(pack.logos.len(), 4);
    for logo in &pack.logos {
        assert_eq!(json["logos"][dokime_organization::variant_name(logo.variant)]["src"], logo.data_uri());
        assert_eq!(logo.alt.as_deref(), Some("Bench Company"));
    }
    assert!(pack.stylesheet().unwrap().contains("data:font/woff2;base64,"));
    assert!(pack.stylesheet().unwrap().contains("--brand-radius-sm"));
}

#[test]
fn local_assets_are_inlined_in_every_stylesheet_and_unsafe_urls_are_refused() {
    let directory = repository().join("organization");
    let original = std::fs::read_to_string(directory.join("theme/base.css")).unwrap();
    let mut source = Overlay {
        directory: theme::Directory::new(&directory).unwrap(),
        files: BTreeMap::new(),
    };
    source.files.insert("mark.svg".into(), b"<svg/>".to_vec());
    source.files.insert("theme/base.css".into(), format!("{original}\na {{ background: url(mark.svg); }}\n").into_bytes());
    let authored = theme::load_from(&source).unwrap();
    assert!(authored.base_css.contains("url(\"data:image/svg+xml;base64,PHN2Zy8+\")"));
    for url in ["https://example.com/font.woff2", "../outside.woff2", "/tmp/font.woff2", "C:/font.woff2"] {
        source.files.insert("theme/base.css".into(), format!("{original}\na {{ background: url('{url}'); }}\n").into_bytes());
        assert!(matches!(theme::load_from(&source), Err(theme::Error::UnsafeUrl { .. })), "{url}");
    }
    source.files.insert("theme/base.css".into(), format!("{original}\n/* url(https://example.com/font.woff2) @import */\n").into_bytes());
    assert!(theme::load_from(&source).is_ok(), "CSS comments remain inert");
}
