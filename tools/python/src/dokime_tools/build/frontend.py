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
import hashlib
import subprocess
import sys
import tempfile
from pathlib import Path

# Third Party Imports
# Local Imports
from dokime_tools.package.staging import clear_output, sync_output

from . import organization, paths

# Type Checking Imports

# === Public Exports === #

# === Constants === #

ROOT = Path(__file__).resolve().parents[5]
WEB_ROOT = ROOT / "crates/dokime-gui"
REPORT = ROOT / "crates/dokime-runtime/dist-report/report.html"
STAMP = ROOT / "target/report-template/inputs.sha256"
INPUT_ROOTS = [
    WEB_ROOT / "src",
    WEB_ROOT / "report.html",
    WEB_ROOT / "vite.config.ts",
    WEB_ROOT / "tsconfig.json",
    WEB_ROOT / "package.json",
    ROOT / "organization/organization.toml",
    ROOT / "organization/theme",
    ROOT / "organization/fonts",
    ROOT / "organization/logos",
    WEB_ROOT / "src/generated/organization/default-organization.json",
    WEB_ROOT / "pnpm-lock.yaml",
]

# === Types === #

# === Classes === #

# === Public Functions === #


# The Stamp Records The Inputs And The Committed Template They Produced, So A Template
# Changed By Checkout Or Hand Is Rendered Again Even When The Inputs Did Not Move.
def is_stale(previous: str | None, current: str) -> bool:
    return previous != current


def ensure_report() -> None:
    organization.generate()
    digest = hashlib.sha256()
    for path in sorted({file for root in INPUT_ROOTS for file in _files(root)}):
        digest.update(str(path.relative_to(WEB_ROOT, walk_up=True)).encode())
        digest.update(b"\0")
        digest.update(path.read_bytes())
        digest.update(b"\0")
    inputs = digest.hexdigest()
    previous = STAMP.read_text(encoding="utf-8").strip() if STAMP.exists() else None
    if is_stale(previous, f"{inputs} {_template_digest()}"):
        STAMP.parent.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(prefix="dokime-report-") as directory:
            output = Path(directory)
            render_report(output)
            # Staging Rewrites The Tracked Template Only When Its Bytes Change.
            sync_output(output, REPORT.parent)
        STAMP.write_text(f"{inputs} {_template_digest()}\n", encoding="utf-8")


def render_report(output: Path) -> None:
    subprocess.run(
        ["node", str(WEB_ROOT / "node_modules/vite/bin/vite.js"), "build", "--mode", "report", "--outDir", str(output)],
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
    organization.generate()
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
    organization.generate()
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


def _template_digest() -> str:
    return hashlib.sha256(REPORT.read_bytes()).hexdigest() if REPORT.is_file() else "absent"


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
