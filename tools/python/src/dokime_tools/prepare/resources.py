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
from dokime_tools.build.frontend import REPORT, ROOT, ensure_report, render_report
from dokime_tools.package.staging import stage_file, sync_output

# Type Checking Imports

# === Public Exports === #

# === Constants === #

BRAND = ROOT / "organization"
DISTRIBUTION = ROOT / "crates/dokime-runtime/dist-brand"
BRAND_FOLDERS = ("theme", "fonts", "logos")

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


def check() -> None:
    with tempfile.TemporaryDirectory(prefix="dokime-resources-") as directory:
        output = Path(directory)
        render_report(output)
        _compare(output, REPORT.parent)

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
    return [BRAND / "organization.toml", *(file for folder in BRAND_FOLDERS for file in sorted((BRAND / folder).iterdir()) if file.is_file())]


def _compare(source: Path, destination: Path) -> None:
    expected = {file.relative_to(source): file.read_bytes() for file in source.rglob("*") if file.is_file()}
    _compare_files(expected, destination)


def _compare_files(expected: dict[Path, bytes], destination: Path) -> None:
    actual = {file.relative_to(destination): file.read_bytes() for file in destination.rglob("*") if file.is_file()}
    changed = sorted(str(path) for path in expected.keys() | actual.keys() if expected.get(path) != actual.get(path))
    if changed:
        raise ValueError(
            f"stale runtime resources in {destination}: {', '.join(changed)}; run `mise run prepare:report` and commit the output"
        )


# === Entry Point === #

if __name__ == "__main__":
    main()
