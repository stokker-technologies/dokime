/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Generate the Checked-In Protocol Bindings.

/* === Modules === */

#[path = "../tests/support/protocol.rs"]
mod protocol;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let output = tempfile::tempdir()?;
    protocol::generate(&root, output.path())?;
    for (generated, committed) in [
        ("dokime.v1.rs", "src/v1.rs"),
        ("harmos.v1.rs", "src/harmos/v1.rs"),
        ("descriptors.bin", "src/descriptors.bin"),
        ("reflection.rs", "src/reflection.rs"),
    ] {
        let bytes = std::fs::read(output.path().join(generated))?;
        let destination = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(committed);
        if std::fs::read(&destination).ok().as_deref() != Some(bytes.as_slice()) {
            std::fs::write(destination, bytes)?;
        }
    }
    for (generated, committed) in [
        ("python/documents.pb", "sdk/python/python/dokime/v1/documents.pb"),
        ("csharp/Documents.cs", "sdk/csharp/Dokime/V1/Documents.cs"),
        ("csharp/Values.cs", "sdk/csharp/Dokime/V1/Values.cs"),
        ("cpp/dokime/v1/documents.pb.h", "sdk/cpp/include/dokime/v1/documents.pb.h"),
        ("cpp/dokime/v1/values.pb.h", "sdk/cpp/include/dokime/v1/values.pb.h"),
        ("cpp/dokime/v1/documents.pb.cc", "sdk/cpp/src/dokime/v1/documents.pb.cc"),
        ("cpp/dokime/v1/values.pb.cc", "sdk/cpp/src/dokime/v1/values.pb.cc"),
    ] {
        let destination = root.join(committed);
        std::fs::create_dir_all(destination.parent().expect("output parent"))?;
        let bytes = std::fs::read(output.path().join(generated))?;
        if std::fs::read(&destination).ok().as_deref() != Some(bytes.as_slice()) {
            std::fs::write(destination, bytes)?;
        }
    }
    Ok(())
}

/* === Tests === */
