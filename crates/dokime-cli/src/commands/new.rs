/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Create the Python Backend Starter

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
    ("plugin/__init__.py", include_bytes!("../../../../sdk/python/examples/starter/plugin/__init__.py")),
    ("plugin/operations/read.py", include_bytes!("../../../../sdk/python/examples/starter/plugin/operations/read.py")),
    ("plugin/suites/capture.py", include_bytes!("../../../../sdk/python/examples/starter/plugin/suites/capture.py")),
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
            fs::write(destination, bytes)?;
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
    terminal.hint("uv add <matching native dokime_python wheel>");
    terminal.hint("uv run python main.py");
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
