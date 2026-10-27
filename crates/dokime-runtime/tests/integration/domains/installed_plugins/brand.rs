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
use std::{fs, path::Path};

// Third Party Imports
use dokime_runtime::domains::{ThemeActivationPolicy, ThemeCornerRadii, ThemeModePolicy};
use tempfile::tempdir;

// Local Imports
use dokime_runtime::domains::installed_plugins::load_brand;

/* === Constants === */
const NEUTRAL_BRAND: &str = r##"
[app]
label = "Acme Test Hub"
slug = "acme"
identifier = "com.acme.hub"
deep_link_scheme = "acme-hub"

[app.organization]
name = "Acme Corp"
unit = "Quality"

[app.logos.transparent]
on_light = "transparent-on-light.svg"
on_dark = "transparent-on-dark.svg"
[app.logos.contained]
on_light = "contained-on-light.svg"
on_dark = "contained-on-dark.svg"

[theme]
policy = "required"
default_mode = "force_dark"
label = "Acme"
description = "Acme brand theme."

[theme.corner_radii]
base = "0.375rem"
sm = "0.125rem"
md = "0.25rem"
lg = "0.375rem"
xl = "0.5rem"
2xl = "0.75rem"
3xl = "1rem"
4xl = "1.25rem"
"##;

const REQUIRED_TEST_TYPOGRAPHY: &[&str] = &[
    "brand-font-sans",
    "brand-font-heading",
    "brand-font-mono",
    "brand-text-xs",
    "brand-text-xl",
    "brand-text-2xl",
    "brand-text-3xl",
    "brand-text-4xl",
    "brand-text-5xl",
    "brand-text-6xl",
    "brand-text-7xl",
];
const REQUIRED_TEST_PALETTE: &[&str] = &[
    "background",
    "foreground",
    "card",
    "card-foreground",
    "popover",
    "popover-foreground",
    "primary",
    "primary-foreground",
    "secondary",
    "secondary-foreground",
    "muted",
    "muted-foreground",
    "accent",
    "accent-foreground",
    "destructive",
    "success",
    "success-foreground",
    "warning",
    "warning-foreground",
    "info",
    "info-foreground",
    "border",
    "input",
    "ring",
];

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */
fn write_brand(dir: &Path, manifest: &str) {
    fs::write(dir.join("organization.toml"), manifest).unwrap();
    let logos = dir.join("logos");
    fs::create_dir_all(&logos).unwrap();
    for variant in ["transparent-on-light", "transparent-on-dark", "contained-on-light", "contained-on-dark"] {
        fs::write(logos.join(format!("{variant}.svg")), format!("<svg id='{variant}'/>")).unwrap();
    }
    fs::create_dir_all(dir.join("theme")).unwrap();
    let typography = REQUIRED_TEST_TYPOGRAPHY.iter().map(|name| format!("--{name}: sample;")).collect::<Vec<_>>().join("\n");
    fs::write(dir.join("theme/typography.css"), format!(":root {{ {typography} }}")).unwrap();
    fs::write(dir.join("theme/base.css"), "::selection { color: currentColor; }").unwrap();
    let palette = |selector: &str, color: &str| {
        let semantic = REQUIRED_TEST_PALETTE
            .iter()
            .map(|name| format!("--{name}: {color};"))
            .chain((1..=8).map(|index| format!("--chart-{index}: {color};")))
            .collect::<Vec<_>>()
            .join("\n");
        format!("{selector} {{ {semantic} }}")
    };
    fs::write(dir.join("theme/light.css"), palette(":root", "#111111")).unwrap();
    fs::write(dir.join("theme/dark.css"), palette(".dark", "#eeeeee")).unwrap();
}

/* === Tests === */

#[test]
fn generated_organization_stylesheet_matches_runtime_resolution() {
    let directory = std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").expect("nextest sets the manifest directory"))
        .join("../../organization");
    let resolved = load_brand(&directory).unwrap();
    let stylesheet = resolved.theme.unwrap().stylesheet.unwrap();
    let generated = fs::read_to_string(directory.join("generated/theme.css")).unwrap();

    assert_eq!(generated, stylesheet);
}

#[test]
fn organization_toml_types() {
    let dir = tempdir().unwrap();
    write_brand(dir.path(), NEUTRAL_BRAND);

    let brand = load_brand(dir.path()).unwrap();

    assert_eq!(brand.identity.label, "Acme Test Hub");
    assert_eq!(brand.identity.slug, "acme");
    assert_eq!(brand.identity.identifier, "com.acme.hub");
    assert_eq!(brand.identity.deep_link_scheme.as_deref(), Some("acme-hub"));

    let customization = &brand.customization;
    assert_eq!(customization.display_name.as_deref(), Some("Acme Test Hub"));
    assert_eq!(customization.title_template.as_deref(), Some("{app}"));
    let organization = customization.organization.as_ref().unwrap();
    assert_eq!(organization.name, "Acme Corp");
    assert_eq!(organization.unit.as_deref(), Some("Quality"));
    assert_eq!(customization.logo_assets.len(), 4);
    assert_eq!(customization.logo_assets[0].asset_id.as_str(), "brand.logo.transparent-on-light");
    assert!(Path::new(&customization.logo_assets[0].path).ends_with(Path::new("logos").join("transparent-on-light.svg")));
    assert_eq!(customization.logo_assets[3].variant, dokime_domain::model::appearance::wire::LogoVariant::ContainedOnDark);

    assert!(matches!(customization.theme_policy, ThemeActivationPolicy::Required { mode: ThemeModePolicy::ForceDark, .. }));

    let theme = brand.theme.expect("theme present");
    assert_eq!(theme.metadata.identifier.as_str(), "brand.theme");
    assert_eq!(theme.metadata.label, "Acme");
    assert_eq!(theme.variables.get("brand-font-sans").map(String::as_str), Some("sample"));
    assert_eq!(
        theme.corner_radii,
        ThemeCornerRadii {
            base: Some("0.375rem".to_owned()),
            sm: Some("0.125rem".to_owned()),
            md: Some("0.25rem".to_owned()),
            lg: Some("0.375rem".to_owned()),
            xl: Some("0.5rem".to_owned()),
            two_xl: Some("0.75rem".to_owned()),
            three_xl: Some("1rem".to_owned()),
            four_xl: Some("1.25rem".to_owned()),
        }
    );
    assert_eq!(theme.light.plot_palette.len(), 8);
    assert_eq!(theme.dark.variables.get("primary").map(String::as_str), Some("#eeeeee"));
}

#[test]
fn app_brand_label() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("organization.toml"),
        r#"
[app]
label = "Minimal"
slug = "minimal"
identifier = "com.minimal.hub"
"#,
    )
    .unwrap();

    let brand = load_brand(dir.path()).unwrap();

    assert_eq!(brand.customization.display_name.as_deref(), Some("Minimal"));
    assert!(brand.customization.logo_assets.is_empty());
    assert!(brand.theme.is_none());
    assert!(matches!(brand.customization.theme_policy, ThemeActivationPolicy::RegisterOnly));
}

#[test]
fn missing_logo_failing() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("organization.toml"),
        r#"
[app]
label = "NoLogo"
slug = "nologo"
identifier = "com.nologo.hub"

[app.logos.contained]
on_light = "missing.svg"
"#,
    )
    .unwrap();

    assert!(load_brand(dir.path()).is_err());
}

#[tokio::test]
async fn runtime_appearance_brand() {
    use dokime_runtime::Runtime;
    use dokime_runtime::{Identifier, Metadata};

    let root = tempdir().unwrap();
    write_brand(root.path(), NEUTRAL_BRAND);
    let brand = load_brand(root.path()).unwrap();
    let mut owner = Runtime::start(crate::service_support::budgets::runtime_config(
        root.path().join("runtime"),
        Metadata::new(Identifier::new("fixture.appearance").unwrap(), "Appearance", None),
    ))
    .await
    .unwrap();
    let runtime = owner.handle();
    runtime.set_brand(Some(brand));

    let frame = runtime.appearance_frame().await;
    assert_eq!(frame.app_name, "Acme Test Hub");
    let captured = runtime.capture_appearance(64 * 1024).unwrap();
    assert_eq!(captured.frame, frame);
    assert_eq!(frame.logos.len(), 4);
    for (selected, captured_asset) in frame.logos.iter().zip(captured.logos.iter()) {
        let asset = runtime.plugin_appearance_asset(&selected.plugin_id, &selected.asset_id).await;
        assert_eq!(captured_asset, &asset);
        assert!(asset.unwrap().path.is_file());
    }

    runtime.set_brand(None);
    assert_ne!(runtime.appearance_frame().await.app_name, "Acme Test Hub");
    drop(captured);
    drop(runtime);
    owner.shutdown().await.unwrap();
}
