/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use dokime_runtime::config::ApplicationOptions;

// an explicit application root keeps settings, plugins and default data local.

#[test]
fn explicit_root_local() {
    let root = tempfile::tempdir().unwrap();
    let config = ApplicationOptions {
        application_root: Some(root.path().to_owned()),
        ..Default::default()
    }
    .resolve()
    .unwrap();
    assert_eq!(config.output_root, root.path().join("data"));
    assert_eq!(config.application_root.as_deref(), Some(root.path()));
    assert_eq!(config.plugin_lookup, None);
    assert!(!root.path().join("settings.json").exists());
    assert!(!root.path().join("config.toml").exists());
    let overridden = ApplicationOptions {
        application_root: Some(root.path().to_owned()),
        storage_root: Some(root.path().join("override")),
        ..Default::default()
    };
    assert_eq!(overridden.resolve().unwrap().output_root, root.path().join("override"));
    assert!(!config.output_root.exists(), "resolution must not start the backend");
}

// a data root keeps one identity across launches, so its first and later
// opens resolve and reopen the same storage.

#[tokio::test]
async fn data_root_reopens() {
    let root = tempfile::tempdir().unwrap();
    let options = ApplicationOptions {
        application_root: Some(root.path().to_owned()),
        ..Default::default()
    };
    let first = options.clone().resolve().unwrap().output_root;
    assert!(!first.exists(), "the first launch creates the data root");
    for launch in 0..3 {
        // Resolve The Same Options The Way Every Shell Launch Does
        let mut config = options.clone().resolve().unwrap();
        assert_eq!(config.output_root, first, "launch {launch} resolved a different data root form");
        #[cfg(windows)]
        assert!(
            !matches!(
                config.output_root.components().next(),
                Some(std::path::Component::Prefix(prefix)) if prefix.kind().is_verbatim()
            ),
            "launch {launch} resolved a verbatim data root: {}",
            config.output_root.display()
        );
        config.seed_plugins = false;

        // Open, Prove Readiness, And Close The Root
        let mut runtime = dokime_runtime::app::launch(config).await.unwrap_or_else(|error| panic!("launch {launch} failed: {error}"));
        let overview = runtime.handle().runtime_overview().await.unwrap();
        assert!(!overview.shutting_down, "launch {launch} accepts commands");
        runtime.shutdown().await.unwrap();
    }
}
