/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

#[cfg(windows)]
use std::os::windows::process::CommandExt;
use std::path::Path;
use std::process::Command;

fn isolate_mise(command: &mut Command, plugin: &Path) -> tempfile::NamedTempFile {
    let configuration = tempfile::Builder::new().prefix("dokime-mise-").suffix(".toml").tempfile().unwrap();
    let ceiling = plugin.parent().expect("the standalone plugin has a parent");
    // Mise Compares The Boundary With Its Non-Verbatim Working Directory
    #[cfg(windows)]
    let ceiling = {
        let path = ceiling.to_str().expect("mise requires a Unicode configuration path");
        match path.strip_prefix(r"\\?\") {
            Some(path) => match path.strip_prefix(r"UNC\") {
                Some(path) => format!(r"\\{path}"),
                None => path.to_owned(),
            },
            None => path.to_owned(),
        }
    };
    command
        .env("MISE_GLOBAL_CONFIG_FILE", configuration.path())
        .env("MISE_SYSTEM_CONFIG_FILE", configuration.path())
        .env("MISE_CEILING_PATHS", ceiling)
        .env_remove("MISE_LOCKED")
        .env_remove("MISE_ENABLE_TOOLS");
    configuration
}

#[test]
fn mise_ignores_machine_tools() {
    let parent = super::support::standalone_directory();
    let plugin = parent.path().join("plugin");
    std::fs::create_dir(&plugin).unwrap();
    std::fs::write(plugin.join("mise.toml"), "[tools]\nnode = \"26.11.1\"\n").unwrap();
    std::fs::write(parent.path().join("mise.toml"), "[tools]\n'gitlab:invalid/ancestor-private' = '0.0.0'\n").unwrap();
    let global = tempfile::Builder::new().suffix(".toml").tempfile().unwrap();
    let system = tempfile::Builder::new().suffix(".toml").tempfile().unwrap();
    std::fs::write(global.path(), "[tools]\n'gitlab:invalid/global-private' = '0.0.0'\n").unwrap();
    std::fs::write(system.path(), "[tools]\n'gitlab:invalid/system-private' = '0.0.0'\n").unwrap();
    let mut command = Command::new("mise");
    #[cfg(windows)]
    command.creation_flags(0x0800_0000);
    command
        .args(["ls", "--current", "--json"])
        .current_dir(&plugin)
        .env("MISE_GLOBAL_CONFIG_FILE", global.path())
        .env("MISE_SYSTEM_CONFIG_FILE", system.path())
        .env("MISE_TRUSTED_CONFIG_PATHS", parent.path())
        .env("MISE_OFFLINE", "true")
        .env("MISE_LOCKED", "1")
        .env("MISE_ENABLE_TOOLS", "python");
    let _configuration = isolate_mise(&mut command, &plugin);
    let output = command.output().expect("mise resolves tools without installation");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let tools: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(tools["node"][0]["version"], "26.11.1", "the starter's own configuration remains visible");
    assert_eq!(tools.as_object().unwrap().len(), 1, "unrelated private tools must not enter the resolution: {tools}");
}
