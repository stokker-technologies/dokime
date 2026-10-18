# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Generate Branding Assets"""

# === Imports === #

# Standard Library Imports
import argparse
import base64
import hashlib
import json
import shutil
import subprocess
import tempfile
import tomllib
from pathlib import Path

# Third Party Imports

# Local Imports

# Type Checking Imports


# === Public Exports === #

# === Constants === #

ROOT = Path(__file__).resolve().parents[5]
BRAND = ROOT / "branding"

# === Types === #

# === Classes === #

# === Public Functions === #


def generate() -> None:
    """Generate The Offline Default Brand From The Runtime Manifest."""
    app = tomllib.loads((BRAND / "brand.toml").read_text())["app"]
    logos = {}
    for treatment in ("transparent", "contained"):
        for surface in ("light", "dark"):
            filename = app.get("logos", {}).get(treatment, {}).get(f"on_{surface}")
            if filename is None:
                continue
            if Path(filename).name != filename or not filename.endswith(".svg"):
                raise ValueError("brand logos must be SVG filenames within branding/logos")
            data = base64.b64encode((BRAND / "logos" / filename).read_bytes()).decode("ascii")
            logos[f"{treatment}_on_{surface}"] = {"src": f"data:image/svg+xml;base64,{data}", "alt": app["label"]}
    _write_json(
        BRAND / "generated/default-brand.json",
        {
            "label": app["label"],
            "title": app.get("title_template", "{app}").replace("{app}", app["label"]),
            "logos": logos,
        },
    )


def generate_icons() -> None:
    """Generate Desktop Icons From The Brand's PNG Icon Master."""
    scratch = ROOT / "target/tmp"
    scratch.mkdir(parents=True, exist_ok=True)
    destination = ROOT / "crates/dokime-gui/src-tauri/icons"
    cli = ROOT / "crates/dokime-gui/node_modules/@tauri-apps/cli/tauri.js"
    with tempfile.TemporaryDirectory(prefix="branding-icons-", dir=scratch) as directory:
        command = ["node", str(cli), "icon", str(BRAND / "icons/icon.png"), "--output", directory]
        subprocess.run(command, cwd=ROOT, check=True)
        subprocess.run([*command, "--png", "64"], cwd=ROOT, check=True)
        outputs = sorted(path for path in Path(directory).iterdir() if path.is_file())
        for path in outputs:
            shutil.copyfile(path, destination / path.name)
        _write_json(
            BRAND / "generated/icon-integrity.json",
            {
                "algorithm": "sha256",
                "source": _digest(BRAND / "icons/icon.png"),
                "outputs": [_digest(destination / path.name) for path in outputs],
            },
        )


# === Private Functions === #


def _write_json(path: Path, value: object) -> None:
    text = json.dumps(value, indent=2) + "\n"
    if not path.exists() or path.read_text() != text:
        path.write_text(text)


def _digest(path: Path) -> dict[str, str]:
    return {"path": path.relative_to(ROOT).as_posix(), "sha256": hashlib.sha256(path.read_bytes()).hexdigest()}


# === Entry Point === #

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--icons", action="store_true", help="Also Regenerate Desktop Icons")
    args = parser.parse_args()
    generate()
    if args.icons:
        generate_icons()
