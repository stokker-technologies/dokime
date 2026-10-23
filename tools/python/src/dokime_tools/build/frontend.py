# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Prepare Frontend And Report Assets"""

# === Imports === #

# Standard Library Imports
import argparse
import subprocess
import sys
import tempfile
from pathlib import Path

# Third Party Imports
# Local Imports
from dokime_tools.build import paths
from dokime_tools.package.staging import clear_output, sync_output

# Type Checking Imports

# === Public Exports === #

# === Constants === #

ROOT = Path(__file__).resolve().parents[5]
WEB_ROOT = ROOT / "crates/dokime-gui"
REPORT = ROOT / "crates/dokime-runtime/dist-report/report.html"

# === Types === #

# === Classes === #

# === Public Functions === #


def ensure_report() -> None:
    with tempfile.TemporaryDirectory(prefix="dokime-report-") as directory:
        output = Path(directory)
        render_report(output)
        sync_output(output, REPORT.parent)


def render_report(output: Path) -> None:
    subprocess.run(
        ["node", str(WEB_ROOT / "node_modules/vite/bin/vite.js"), "build", "--config", "vite.report.config.ts", "--outDir", str(output)],
        cwd=WEB_ROOT,
        check=True,
        creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
        stdin=sys.stdin,
        stdout=sys.stdout,
        stderr=sys.stderr,
    )
    if not (output / REPORT.name).is_file():
        raise ValueError(f"report build did not produce {REPORT.name}")


def build_live(args: list[str]) -> None:
    vite = WEB_ROOT / "node_modules/vite/bin/vite.js"
    command = ["node", str(vite), "build"]
    if args:
        subprocess.run(
            command + args,
            cwd=WEB_ROOT,
            check=True,
            creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
            stdin=sys.stdin,
            stdout=sys.stdout,
            stderr=sys.stderr,
        )
        return
    with tempfile.TemporaryDirectory(prefix="dokime-gui-live-") as directory:
        subprocess.run(
            command + ["--outDir", directory],
            cwd=WEB_ROOT,
            check=True,
            creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
            stdin=sys.stdin,
            stdout=sys.stdout,
            stderr=sys.stderr,
        )
        sync_output(Path(directory), WEB_ROOT / "dist")


def build(profile: str) -> Path:
    """Build The Live Frontend Once And Mirror It Where The Desktop Shell Serves It From."""
    output = paths.output(ROOT, "build", profile) / "web"
    clear_output(output)
    subprocess.run(
        ["node", str(WEB_ROOT / "node_modules/vite/bin/vite.js"), "build", "--outDir", str(output)],
        cwd=WEB_ROOT,
        check=True,
        creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
        stdin=sys.stdin,
        stdout=sys.stdout,
        stderr=sys.stderr,
    )
    sync_output(output, WEB_ROOT / "dist")
    return output


def check_assets() -> None:
    if not REPORT.is_file():
        raise ValueError(f"missing generated asset: {REPORT}")
    for root in [WEB_ROOT / "dist", REPORT.parent]:
        if not root.is_dir():
            raise ValueError(f"missing generated asset directory: {root}")
        for path in _files(root):
            if path.suffix in {".css", ".html", ".js", ".json", ".svg", ".txt"}:
                text = path.read_text(encoding="utf-8")
                if any(pattern in text for pattern in ["WiFi Band Usage", "WifiBandUsage", "wifi_nearby_"]):
                    raise ValueError(f"generated asset contains removed WiFi band usage code: {path}")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=["report", "live", "assets"])
    args, extra = parser.parse_known_args()
    if args.command != "live" and extra:
        parser.error(f"unexpected arguments: {extra}")
    if args.command == "report":
        ensure_report()
    elif args.command == "live":
        build_live(extra)
    elif args.command == "assets":
        check_assets()


# === Private Functions === #


def _files(root: Path) -> list[Path]:
    if root.is_file():
        return [root]
    if not root.exists():
        return []
    files = []
    for path in root.iterdir():
        if path.name == "node_modules" or path.name.startswith("."):
            continue
        files.extend(_files(path))
    return files


# === Entry Point === #

if __name__ == "__main__":
    main()
