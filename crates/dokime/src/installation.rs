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
use std::path::{Path, PathBuf};
use std::process::Command;

// Third Party Imports

// Local Imports
use crate::client_error::{Error, Result};

/* === Constants === */

#[cfg(any(windows, target_os = "macos"))]
const PRODUCT: &str = "Dokime";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct Program {
    name: &'static str,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Program {
    pub const SERVER: Self = Self { name: "dokime-server" };
    pub const GUI: Self = Self { name: "dokime-gui" };

    pub fn resolve(&self, explicit: Option<&Path>) -> Result<PathBuf> {
        if let Some(path) = explicit {
            return executable(path).ok_or_else(|| Error::Input(format!("{} {} is not a file", self.name, path.display())));
        }

        let file = format!("{}{}", self.name, std::env::consts::EXE_SUFFIX);
        let beside = std::env::current_exe().ok().and_then(|path| path.parent().map(Path::to_owned));
        let home = std::env::var_os("DOKIME_HOME").map(PathBuf::from);
        beside
            .into_iter()
            .chain(home)
            .chain(installed_directories())
            .find_map(|directory| executable(&directory.join(&file)))
            .ok_or_else(|| {
                Error::Setup(format!(
                    "no {} is installed; install Dokime with the desktop installer, set DOKIME_HOME to its directory, or pass its path explicitly",
                    self.name
                ))
            })
    }
}

/* === Public Functions === */

pub fn hand_over(mut command: Command) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        Err(command.exec().into())
    }
    #[cfg(not(unix))]
    {
        let status = command.status()?;
        std::process::exit(status.code().unwrap_or(1));
    }
}

/* === Private Functions === */

fn executable(path: &Path) -> Option<PathBuf> {
    if !path.is_file() {
        return None;
    }
    std::path::absolute(path).ok()
}

#[cfg(windows)]
fn installed_directories() -> Vec<PathBuf> {
    ["LOCALAPPDATA", "ProgramFiles"].into_iter().filter_map(std::env::var_os).map(|root| PathBuf::from(root).join(PRODUCT)).collect()
}

#[cfg(target_os = "macos")]
fn installed_directories() -> Vec<PathBuf> {
    vec![PathBuf::from(format!("/Applications/{PRODUCT}.app/Contents/MacOS"))]
}

#[cfg(not(any(windows, target_os = "macos")))]
fn installed_directories() -> Vec<PathBuf> {
    vec![PathBuf::from("/usr/bin"), PathBuf::from("/usr/local/bin")]
}

/* === Tests === */
