/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Build Plugin Projects Before Use

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{io::ErrorKind, path::Path, process::Stdio};

// Third Party Imports
use tokio::process::Command;

// Local Imports
use crate::{Result, error::Error};
use dokime_packaging::project::PluginProject;
use dokime_runtime::PluginProcess;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(in crate::commands) async fn prepare(root: &Path) -> Result<PluginProject> {
    let project = PluginProject::read(root)?;
    if let Some(build) = &project.manifest.build {
        let command = PluginProcess::command(&project, build)?;
        let status = Command::from(command).stdin(Stdio::null()).kill_on_drop(true).status().await.map_err(|source| {
            if source.kind() == ErrorKind::NotFound {
                Error::Invalid(format!("Build Plugin: Install {} and Put It on PATH, Then Retry", build[0]))
            } else {
                Error::Invalid(format!("Build Plugin: Correct the build Command in dokime.toml: {source}"))
            }
        })?;
        if !status.success() {
            return Err(format!("Build Plugin: {} Exited with {status}; Fix Its Build Errors and Retry", build[0]).into());
        }
    }
    // Revalidate Paths After a Build That May Have Created or Replaced Them
    let project = PluginProject::read(&project.root)?;
    project.validate_inputs()?;
    Ok(project)
}

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    use std::fs;

    use super::prepare;
    use crate::error::Error;

    #[tokio::test]
    async fn missing_build_program_names_what_to_install() -> crate::Result<()> {
        let root = tempfile::tempdir()?;
        let program = "./absent-build-program";
        fs::write(root.path().join("dokime.toml"), format!("build = ['{program}']\nlaunch = ['fixture']\n"))?;
        let error = prepare(root.path()).await.unwrap_err();
        assert!(matches!(error, Error::Invalid(ref message) if message.contains(program)));
        Ok(())
    }
}
