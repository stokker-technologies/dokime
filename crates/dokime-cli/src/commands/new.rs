/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Create the Python Plugin Starter

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{fs, path::PathBuf};

// Third Party Imports

// Local Imports

/* === Constants === */

const FILES: &[(&str, &[u8])] = &[
    (".gitattributes", include_bytes!("../../../../sdk/python/examples/starter/.gitattributes")),
    (".gitignore", include_bytes!("../../../../sdk/python/examples/starter/.gitignore")),
    ("pyproject.toml", include_bytes!("../../../../sdk/python/examples/starter/pyproject.toml")),
    ("main.py", include_bytes!("../../../../sdk/python/examples/starter/main.py")),
    ("suite.py", include_bytes!("../../../../sdk/python/examples/starter/suite.py")),
    ("mise.toml", include_bytes!("../../../../sdk/python/examples/starter/mise.toml")),
    ("tests/host.py", include_bytes!("../../../../sdk/python/examples/starter/tests/host.py")),
    ("tests/test_capture.py", include_bytes!("../../../../sdk/python/examples/starter/tests/test_capture.py")),
];

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Debug, clap::Args)]
pub(crate) struct Args {
    #[arg(help = "New project directory. Its parent must already exist.")]
    path: PathBuf,

    #[arg(long, value_enum, help = "Project authoring language.")]
    lang: Language,
}

/* === Enums === */

#[derive(Clone, Copy, Debug, clap::ValueEnum)]
enum Language {
    Python,
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) fn run(args: Args) -> crate::Result<()> {
    // Resolve the Project Name
    let name = args
        .path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| crate::Error::Input("project directory must have a UTF-8 name".into()))?;
    let name = serde_json::to_string(name)?;

    // Reserve the Destination
    fs::create_dir(&args.path)?;

    // Write the Embedded Starter
    let files = match args.lang {
        Language::Python => FILES,
    };
    let result = (|| -> std::io::Result<()> {
        for (relative, bytes) in files {
            let destination = args.path.join(relative);
            fs::create_dir_all(destination.parent().expect("starter file has a parent"))?;
            if *relative == "pyproject.toml" {
                let manifest = std::str::from_utf8(bytes).expect("embedded Python manifest is UTF-8");
                fs::write(destination, manifest.replacen("\"dokime-starter\"", &name, 1))?;
            } else {
                fs::write(destination, bytes)?;
            }
        }
        Ok(())
    })();
    if let Err(error) = result {
        fs::remove_dir_all(&args.path)?;
        return Err(error.into());
    }

    // Show the Next Commands
    let terminal = dokime::terminal::Terminal::default();
    terminal.success(format!("Created {}", args.path.display()));
    terminal.hint(format!("cd {}", args.path.display()));
    terminal.hint("mise run build");
    terminal.hint("mise run test");
    terminal.hint("mise run serve");
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
