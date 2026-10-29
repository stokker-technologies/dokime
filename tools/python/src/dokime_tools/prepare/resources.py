# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Refresh Or Check Crate-Local Embedded Runtime Resources"""

# === Imports === #

# Standard Library Imports
import argparse
import tempfile
from pathlib import Path

# Third Party Imports
# Local Imports
from dokime_tools.build.frontend import ROOT, ensure_report
from dokime_tools.package.staging import stage_file, sync_output

# Type Checking Imports

# === Public Exports === #

# === Constants === #

BRAND = ROOT / "organization"
DISTRIBUTION = ROOT / "crates/dokime-runtime/dist-brand"

# === Types === #

# === Classes === #

# === Public Functions === #


def refresh() -> None:
    ensure_report()
    with tempfile.TemporaryDirectory(prefix="dokime-brand-") as directory:
        output = Path(directory)
        for source in _brand_files():
            stage_file(source, output / source.relative_to(BRAND))
        sync_output(output, DISTRIBUTION)


# The Report Template Is Generated Build Output, So Only The Committed Brand Bundle Is Checked.
def check() -> None:
    expected = {source.relative_to(BRAND): source.read_bytes() for source in _brand_files()}
    _compare_files(expected, DISTRIBUTION)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    if args.check:
        check()
    else:
        refresh()


# === Private Functions === #


def _brand_files() -> list[Path]:
    if not (BRAND / "organization.toml").is_file():
        raise ValueError("organization pack has no manifest")

    files = []
    for path in sorted(BRAND.rglob("*")):
        if path.is_symlink():
            raise ValueError("organization packs may not contain links")
        if not path.is_file():
            continue
        files.append(path)
    return files


def _compare_files(expected: dict[Path, bytes], destination: Path) -> None:
    actual = {file.relative_to(destination): file.read_bytes() for file in destination.rglob("*") if file.is_file()}
    changed = sorted(str(path) for path in expected.keys() | actual.keys() if expected.get(path) != actual.get(path))
    if changed:
        raise ValueError(
            f"stale runtime resources in {destination}: {', '.join(changed)}; run `mise run prepare:report` and commit the brand bundle"
        )


# === Entry Point === #

if __name__ == "__main__":
    main()
