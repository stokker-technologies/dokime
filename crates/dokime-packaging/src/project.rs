/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Read and Validate Plugin Project Manifests

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    env::consts::EXE_SUFFIX,
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};

// Third Party Imports
use harmos_runtime::bundle::validate_launch;
use serde::Deserialize;

// Local Imports
use crate::Result;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PluginManifest {
    pub build: Option<Vec<String>>,
    pub executable: Option<PathBuf>,
    pub launch: Option<Vec<String>>,

    #[serde(default)]
    pub assets: Vec<PathBuf>,
}

#[derive(Debug)]
pub struct PluginProject {
    pub root: PathBuf,
    pub manifest: PluginManifest,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl PluginProject {
    pub fn read(root: &Path) -> Result<Self> {
        let root = root.canonicalize()?;
        let source = fs::read_to_string(root.join("dokime.toml"))
            .map_err(|source| format!("Read Plugin Project: Add dokime.toml in {}: {source}", root.display()))?;
        let manifest =
            toml::from_str::<PluginManifest>(&source).map_err(|source| format!("Read Plugin Project: Correct dokime.toml: {source}"))?;
        if manifest.executable.is_some() == manifest.launch.is_some() {
            return Err("Read Plugin Project: Set Exactly One of executable or launch in dokime.toml".into());
        }
        if let Some(build) = &manifest.build {
            validate_launch(build).map_err(|source| format!("Read Plugin Project: Correct build in dokime.toml: {source}"))?;
        }
        if let Some(launch) = &manifest.launch {
            validate_launch(launch).map_err(|source| format!("Read Plugin Project: Correct launch in dokime.toml: {source}"))?;
        }
        let project = Self { root, manifest };
        if let Some(executable) = &project.manifest.executable {
            project.path(executable)?;
        }
        for asset in &project.manifest.assets {
            project.path(asset)?;
        }
        for command in [&project.manifest.build, &project.manifest.launch].into_iter().flatten() {
            if command[0].contains(['/', '\\']) {
                project.path(Path::new(&command[0]))?;
            }
        }
        Ok(project)
    }

    pub fn validate_inputs(&self) -> Result<()> {
        let python = self.root.join("pyproject.toml").exists()
            || self.manifest.launch.as_ref().is_some_and(|command| {
                command.iter().any(|argument| argument == "python" || argument == "python3" || argument.ends_with(".py"))
            });
        if python && (!self.root.join("pyproject.toml").is_file() || !self.root.join("uv.lock").is_file()) {
            return Err(
                "Read Python Plugin Project: Add pyproject.toml and uv.lock to the Plugin Root; Run uv lock to Create the Lock".into()
            );
        }
        Ok(())
    }

    pub fn executable(&self) -> Result<PathBuf> {
        let relative = self.manifest.executable.as_deref().ok_or("Start Native Plugin: Set executable in dokime.toml")?;
        let mut executable = self.path(relative)?;
        if !executable.is_file() && executable.extension().is_none() && !EXE_SUFFIX.is_empty() {
            executable.set_extension(EXE_SUFFIX.trim_start_matches('.'));
        }
        let executable = executable.canonicalize().map_err(|source| {
            format!("Start Native Plugin: Build {} or Correct executable in dokime.toml: {source}", executable.display())
        })?;
        if !executable.starts_with(&self.root) || !executable.is_file() {
            return Err("Start Native Plugin: Set executable to a File Inside the Plugin Root in dokime.toml".into());
        }
        Ok(executable)
    }

    pub fn path(&self, relative: &Path) -> Result<PathBuf> {
        let text = relative.to_str().ok_or("Read Plugin Project: Use UTF-8 Paths in dokime.toml")?;
        if text.is_empty() || text.starts_with(['/', '\\']) || text.contains(':') {
            return Err("Read Plugin Project: Use Relative Paths Inside the Plugin Root in dokime.toml".into());
        }
        let mut path = PathBuf::new();
        for component in text.split(['/', '\\']) {
            match component {
                "" | "." => {}
                ".." => {
                    if !path.pop() {
                        return Err("Read Plugin Project: Keep Paths Inside the Plugin Root in dokime.toml".into());
                    }
                }
                component => path.push(component),
            }
        }
        let path = self.root.join(path);
        let mut ancestor = path.as_path();
        loop {
            match ancestor.canonicalize() {
                Ok(resolved) => {
                    if !resolved.starts_with(&self.root) {
                        return Err("Read Plugin Project: Keep Symbolic Links Inside the Plugin Root".into());
                    }
                    break;
                }
                Err(source) if source.kind() == ErrorKind::NotFound => {
                    ancestor = ancestor.parent().ok_or("Read Plugin Project: Keep Paths Inside the Plugin Root")?;
                }
                Err(source) => return Err(source.into()),
            }
        }
        Ok(path)
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    use std::{fs, path::Path};

    use super::PluginProject;
    use crate::Result;

    #[test]
    fn manifests_accept_native_and_launch_projects() -> Result<()> {
        let root = tempfile::tempdir()?;
        fs::write(
            root.path().join("dokime.toml"),
            "build = ['cargo', 'build']\nexecutable = 'target/debug/plugin'\nassets = ['assets']\n",
        )?;
        let native = PluginProject::read(root.path())?;
        assert_eq!(native.manifest.build, Some(vec!["cargo".into(), "build".into()]));
        assert_eq!(native.manifest.executable.as_deref(), Some(Path::new("target/debug/plugin")));
        assert_eq!(native.path(Path::new("assets"))?, root.path().canonicalize()?.join("assets"));

        fs::write(root.path().join("dokime.toml"), "launch = ['fixture', 'entry']\n")?;
        let launch = PluginProject::read(root.path())?;
        assert_eq!(launch.manifest.launch, Some(vec!["fixture".into(), "entry".into()]));
        assert!(launch.manifest.build.is_none());
        assert!(launch.manifest.assets.is_empty());
        Ok(())
    }

    #[test]
    fn manifests_refuse_ambiguous_unknown_and_escaping_inputs() -> Result<()> {
        let root = tempfile::tempdir()?;
        for source in [
            "executable = 'plugin'\nlaunch = ['fixture']",
            "assets = []",
            "executable = 'plugin'\nidentity = 'fixture'",
            "executable = '../plugin'",
            "executable = '/plugin'",
            "executable = 'C:\\plugin'",
            "executable = 'plugin'\nassets = ['..\\asset']",
            "launch = ['../fixture']",
            "launch = []",
            "launch = ['']",
            "launch = ['fixture']\nbuild = []",
        ] {
            fs::write(root.path().join("dokime.toml"), source)?;
            assert!(PluginProject::read(root.path()).is_err(), "{source}");
        }
        Ok(())
    }

    #[test]
    fn python_projects_require_both_project_and_lock() -> Result<()> {
        let root = tempfile::tempdir()?;
        fs::write(root.path().join("dokime.toml"), "launch = ['uv', 'run', 'python', 'plugin.py']")?;
        assert!(PluginProject::read(root.path())?.validate_inputs().is_err());
        fs::write(root.path().join("pyproject.toml"), "[project]\nname = 'fixture'\nversion = '0.1.0'")?;
        assert!(PluginProject::read(root.path())?.validate_inputs().is_err());
        fs::write(root.path().join("uv.lock"), "version = 1")?;
        assert!(PluginProject::read(root.path())?.validate_inputs().is_ok());
        Ok(())
    }

    #[test]
    fn native_projects_resolve_the_platform_executable_suffix() -> Result<()> {
        let root = tempfile::tempdir()?;
        fs::write(root.path().join("dokime.toml"), "executable = 'plugin'\n")?;
        let executable = root.path().join(format!("plugin{}", super::EXE_SUFFIX));
        fs::write(&executable, [2, 3, 5])?;
        let project = PluginProject::read(root.path())?;
        assert_eq!(project.executable()?, executable.canonicalize()?);
        Ok(())
    }
}
