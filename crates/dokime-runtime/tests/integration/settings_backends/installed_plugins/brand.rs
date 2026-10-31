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
use base64::Engine as _;
use dokime_runtime::domains::{ThemeActivationPolicy, ThemeCornerRadii, ThemeModePolicy};
use tempfile::tempdir;

// Local Imports
use dokime_runtime::domains::installed_plugins::load_brand;

/* === Constants === */
const NEUTRAL_BRAND: &str = r##"
[app]
label = "Stokker Technologies Test Hub"

[app.organization]
name = "Stokker Technologies"
unit = "Quality"

[app.logos.transparent]
on_light = "logos/transparent-on-light.svg"
on_dark = "logos/transparent-on-dark.svg"
[app.logos.contained]
on_light = "logos/contained-on-light.svg"
on_dark = "logos/contained-on-dark.svg"

[theme]
policy = "required"
default_mode = "force_dark"
label = "Stokker Technologies"
description = "Stokker Technologies brand theme."

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

pub(crate) fn organization_overlay() {
    use dokime_runtime::config::organization::resolve_with_base;

    let root = tempdir().unwrap();
    let baseline = resolve_with_base(None, None).unwrap();
    let installed = root.path().join("installed");
    baseline.write_to(&installed).unwrap();
    let overrides = root.path().join("overrides");
    fs::create_dir_all(overrides.join("logos")).unwrap();
    fs::write(
        overrides.join("organization.toml"),
        "[app]\nlabel = 'Stokker Technologies'\n[app.organization]\nunit = 'Quality'\n[app.logos.transparent]\non_dark = 'icons/icon.png'\n",
    )
    .unwrap();
    let replacement = b"<svg xmlns='http://www.w3.org/2000/svg' id='override'/>";
    fs::write(overrides.join("logos/transparent-on-light.svg"), replacement).unwrap();

    let pack = resolve_with_base(Some(&installed), Some(&overrides)).unwrap();
    assert_eq!(pack.label(), "Stokker Technologies");
    assert_eq!(pack.manifest["app"]["organization"]["unit"].as_str(), Some("Quality"));
    assert_eq!(pack.manifest["app"]["organization"]["name"], baseline.manifest["app"]["organization"]["name"]);
    assert_eq!(pack.files["logos/transparent-on-light.svg"], replacement);
    assert_eq!(pack.files["theme/light.css"], baseline.files["theme/light.css"]);
    assert!(pack.brand().unwrap().theme.unwrap().stylesheet.unwrap().contains("data:font/woff2;base64,"));
    assert!(pack.brand().unwrap().logos.iter().any(|logo| logo.mime_type == "image/png"));

    fs::remove_file(installed.join("theme/dark.css")).unwrap();
    assert!(resolve_with_base(Some(&installed), Some(&overrides)).is_err());
}

pub(crate) fn organization_validation() {
    use dokime_runtime::config::organization::resolve_with_base;

    let overrides = tempdir().unwrap();
    let manifest = overrides.path().join("organization.toml");
    for value in [
        "[app]\nidentifier = 'other.identity'",
        "[app.logos.transparent]\non_light = '../outside.svg'",
        "[app.artwork]\ntray = 'missing.png'",
        "[app.artwork]\ntray = 'logos/transparent-on-light.svg'",
        "[updates]\nmode = 'manual'\nsource = 'missing'",
    ] {
        fs::write(&manifest, value).unwrap();
        assert!(resolve_with_base(None, Some(overrides.path())).is_err());
    }
    fs::write(&manifest, "[app]\nlabel = 'Stokker Technologies'").unwrap();
    fs::create_dir(overrides.path().join("generated")).unwrap();
    assert!(resolve_with_base(None, Some(overrides.path())).is_err());

    let mut pack = resolve_with_base(None, None).unwrap();
    pack.files.insert("../outside".into(), vec![0]);
    let destination = tempdir().unwrap();
    assert!(pack.write_to(&destination.path().join("target")).is_err());
    assert!(!destination.path().join("outside").exists());

    let active = destination.path().join("organization/active");
    fs::create_dir_all(&active).unwrap();
    fs::write(active.join("organization.toml"), "invalid [").unwrap();
    let config = dokime_runtime::config::ApplicationOptions {
        application_root: Some(destination.path().into()),
        ..Default::default()
    }
    .resolve()
    .unwrap();
    assert_eq!(config.brand.unwrap().identity.label, "Dokime");
    assert_eq!(fs::read_to_string(active.join("organization.toml")).unwrap(), "invalid [");
}

pub(crate) fn installer_sidebar_overlay() {
    use dokime_runtime::config::organization::resolve_with_base;

    let baseline = resolve_with_base(None, None).unwrap();
    assert_eq!(baseline.installer_sidebar_color().unwrap(), [0x1b, 0x19, 0x15]);
    assert!(baseline.installer_sidebar_image().is_none());
    let root = tempdir().unwrap();
    let installed = root.path().join("installed");
    let overrides = root.path().join("overrides");
    baseline.write_to(&installed).unwrap();
    fs::create_dir_all(overrides.join("installer")).unwrap();
    let image = baseline.files["icons/icon.png"].clone();
    fs::write(overrides.join("installer/sidebar.png"), &image).unwrap();
    let manifest = overrides.join("organization.toml");
    fs::write(&manifest, "[installer]\nsidebar_image = 'installer/sidebar.png'").unwrap();

    let pack = resolve_with_base(Some(&installed), Some(&overrides)).unwrap();
    assert_eq!(pack.installer_sidebar_color().unwrap(), baseline.installer_sidebar_color().unwrap());
    assert_eq!(pack.installer_sidebar_image(), Some(("installer/sidebar.png", image.as_slice())));
    pack.write_to(&installed).unwrap();
    fs::write(&manifest, "[installer]\nsidebar_color = '#aB01fF'").unwrap();

    let pack = resolve_with_base(Some(&installed), Some(&overrides)).unwrap();
    assert_eq!(pack.installer_sidebar_color().unwrap(), [0xab, 0x01, 0xff]);
    assert_eq!(pack.installer_sidebar_image(), Some(("installer/sidebar.png", image.as_slice())));
    let mut old_manifest = baseline.manifest.clone();
    old_manifest.as_table_mut().unwrap().remove("installer");
    fs::write(installed.join("organization.toml"), toml::to_string(&old_manifest).unwrap()).unwrap();
    let old_pack = resolve_with_base(Some(&installed), None).unwrap();
    assert_eq!(old_pack.installer_sidebar_color().unwrap(), [0x1b, 0x19, 0x15]);
}

pub(crate) fn installer_sidebar_validation() {
    use dokime_runtime::config::organization::resolve_with_base;

    let overrides = tempdir().unwrap();
    let manifest = overrides.path().join("organization.toml");
    let invalid = [
        "installer = 'invalid'",
        "[installer]\nsidebar_color = 123456",
        "[installer]\nsidebar_color = '#123'",
        "[installer]\nsidebar_color = '#12345678'",
        "[installer]\nsidebar_color = '123456'",
        "[installer]\nsidebar_color = '#gg0000'",
        "[installer]\nsidebar_image = 1",
        "[installer]\nsidebar_image = ''",
        "[installer]\nsidebar_image = '../outside.png'",
        "[installer]\nsidebar_image = '/outside.png'",
        "[installer]\nsidebar_image = 'icons/../icon.png'",
        "[installer]\nsidebar_image = 'C:/outside.png'",
        "[installer]\nsidebar_image = 'missing.png'",
        "[installer]\nsidebar_image = 'logos/transparent-on-light.svg'",
        "[installer]\nsidebar_image = 'broken.png'",
        "[installer]\nsidebar_color = 'invalid'\nsidebar_image = 'icons/icon.png'",
    ];
    fs::write(overrides.path().join("broken.png"), b"\x89PNG\r\n\x1a\ntruncated").unwrap();
    for value in invalid {
        fs::write(&manifest, value).unwrap();
        assert!(resolve_with_base(None, Some(overrides.path())).is_err(), "accepted invalid settings: {value}");
    }
}

pub(crate) fn generated_organization_stylesheet_matches_runtime_resolution() {
    let directory = std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").expect("nextest sets the manifest directory"))
        .join("../../organization");
    let resolved = load_brand(&directory).unwrap();
    let mut stylesheet = resolved.theme.unwrap().stylesheet.unwrap();
    let mut generated = fs::read_to_string(directory.join("../crates/dokime-gui/src/generated/organization/theme.css")).unwrap();

    // Compare Font Bytes Across The Generated And Runtime Data URI Encodings.
    for font in fs::read_dir(directory.join("fonts")).unwrap() {
        let path = font.unwrap().path();
        let mime = match path.extension().and_then(|extension| extension.to_str()) {
            Some("woff2") => "font/woff2",
            Some("woff") => "font/woff",
            Some("ttf") => "font/ttf",
            Some("otf") => "font/otf",
            _ => continue,
        };
        let bytes = fs::read(path).unwrap();
        let escaped = bytes.iter().map(|byte| format!("%{byte:02X}")).collect::<String>();
        let encoded = base64::engine::general_purpose::STANDARD.encode(&bytes);
        let generated_uri = format!("data:{mime},{escaped}");
        let runtime_uri = format!("data:{mime};base64,{encoded}");
        generated = generated.replace(&generated_uri, &runtime_uri);
        stylesheet = stylesheet.replace(&generated_uri, &runtime_uri);
    }

    assert_eq!(generated, stylesheet);
}

pub(crate) fn organization_toml_types() {
    let dir = tempdir().unwrap();
    write_brand(dir.path(), NEUTRAL_BRAND);

    let brand = load_brand(dir.path()).unwrap();

    assert_eq!(brand.identity.label, "Stokker Technologies Test Hub");

    let customization = &brand.customization;
    assert_eq!(customization.display_name.as_deref(), Some("Stokker Technologies Test Hub"));
    assert_eq!(customization.title_template.as_deref(), Some("{app}"));
    let organization = customization.organization.as_ref().unwrap();
    assert_eq!(organization.name, "Stokker Technologies");
    assert_eq!(organization.unit.as_deref(), Some("Quality"));
    assert_eq!(customization.logo_assets.len(), 4);
    assert_eq!(customization.logo_assets[0].asset_id.as_str(), "brand.logo.transparent-on-light");
    assert!(Path::new(&customization.logo_assets[0].path).ends_with(Path::new("logos").join("transparent-on-light.svg")));
    assert_eq!(customization.logo_assets[3].variant, dokime_domain::model::appearance::wire::LogoVariant::ContainedOnDark);

    assert!(matches!(customization.theme_policy, ThemeActivationPolicy::Required { mode: ThemeModePolicy::ForceDark, .. }));

    let theme = brand.theme.expect("theme present");
    assert_eq!(theme.metadata.identifier.as_str(), "brand.theme");
    assert_eq!(theme.metadata.label, "Stokker Technologies");
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

pub(crate) fn app_brand_label() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("organization.toml"),
        r#"
[app]
label = "Stokker Technologies Minimal"
"#,
    )
    .unwrap();

    let brand = load_brand(dir.path()).unwrap();

    assert_eq!(brand.customization.display_name.as_deref(), Some("Stokker Technologies Minimal"));
    assert!(brand.customization.logo_assets.is_empty());
    assert!(brand.theme.is_none());
    assert!(matches!(brand.customization.theme_policy, ThemeActivationPolicy::RegisterOnly));
}

pub(crate) fn missing_logo_failing() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("organization.toml"),
        r#"
[app]
label = "Stokker Technologies Without Artwork"

[app.logos.contained]
on_light = "missing.svg"
"#,
    )
    .unwrap();

    assert!(load_brand(dir.path()).is_err());
}

pub(crate) async fn runtime_appearance_brand() {
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
    assert_eq!(frame.app_name, "Stokker Technologies Test Hub");
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
