/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let source = PathBuf::from(args.next().ok_or("expected organization directory")?);
    let output = PathBuf::from(args.next().ok_or("expected output directory")?);
    if args.next().is_some() {
        return Err("expected organization and output directories".into());
    }
    let directory = dokime_organization::theme::Directory::new(&source)?;
    let text = std::fs::read_to_string(source.join("organization.toml"))?;
    let pack = dokime_organization::Materialized::read(&text, &directory)?;
    std::fs::create_dir_all(&output)?;
    for (name, bytes) in [("default-organization.json", pack.default_json()?), ("theme.css", pack.stylesheet().unwrap_or_default())] {
        let path = output.join(name);
        if std::fs::read(&path).ok().as_deref() != Some(bytes.as_bytes()) {
            std::fs::write(path, bytes)?;
        }
    }
    Ok(())
}
