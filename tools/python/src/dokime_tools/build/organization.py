# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Generate Organization Assets"""

# === Imports === #

# Standard Library Imports
import argparse
import base64
import hashlib
import json
import re
import shutil
import subprocess
import sys
import tempfile
import tomllib
from pathlib import Path

# Third Party Imports

# Local Imports

# Type Checking Imports


# === Public Exports === #

# === Constants === #

ROOT = Path(__file__).resolve().parents[5]
BRAND = ROOT / "organization"
GENERATED = ROOT / "crates/dokime-gui/src/generated/organization"

# === Types === #

# === Classes === #

# === Public Functions === #


def generate() -> None:
    """Generate The Offline Default Brand From The Runtime Manifest."""
    manifest = tomllib.loads((BRAND / "organization.toml").read_text())
    app = manifest["app"]
    logos = {}
    for treatment in ("transparent", "contained"):
        for surface in ("light", "dark"):
            filename = app.get("logos", {}).get(treatment, {}).get(f"on_{surface}")
            if filename is None:
                continue
            path = (BRAND / filename).resolve()
            mime = {".svg": "image/svg+xml", ".png": "image/png"}.get(path.suffix)
            if not path.is_relative_to(BRAND.resolve()) or mime is None:
                raise ValueError("brand logos must be pack-root-relative SVG or PNG paths")
            data = base64.b64encode(path.read_bytes()).decode("ascii")
            logos[f"{treatment}_on_{surface}"] = {"src": f"data:{mime};base64,{data}", "alt": app["label"]}
    _write_json(
        GENERATED / "default-organization.json",
        {
            "label": app["label"],
            "title": app.get("title_template", "{app}").replace("{app}", app["label"]),
            "logos": logos,
        },
    )

    _write_text(GENERATED / "theme.css", _stylesheet(manifest))


def generate_icons() -> None:
    """Generate Desktop Icons From The Brand's PNG Icon Master."""
    destination = ROOT / "crates/dokime-gui/src-tauri/icons"
    cli = ROOT / "crates/dokime-gui/node_modules/@tauri-apps/cli/tauri.js"
    with tempfile.TemporaryDirectory(prefix="organization-icons-") as directory:
        command = ["node", str(cli), "icon", str(BRAND / "icons/icon.png"), "--output", directory]
        subprocess.run(
            command,
            cwd=ROOT,
            check=True,
            creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
            stdin=sys.stdin,
            stdout=sys.stdout,
            stderr=sys.stderr,
        )
        subprocess.run(
            [*command, "--png", "64"],
            cwd=ROOT,
            check=True,
            creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
            stdin=sys.stdin,
            stdout=sys.stdout,
            stderr=sys.stderr,
        )
        outputs = sorted(path for path in Path(directory).iterdir() if path.is_file())
        for path in outputs:
            shutil.copyfile(path, destination / path.name)
        _write_json(
            destination / "icon-integrity.json",
            {
                "algorithm": "sha256",
                "source": _digest(BRAND / "icons/icon.png"),
                "outputs": [_digest(destination / path.name) for path in outputs],
            },
        )


# === Private Functions === #


def _write_json(path: Path, value: object) -> None:
    text = json.dumps(value, indent=2) + "\n"
    _write_text(path, text)


def _write_text(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    encoded = text.encode("utf-8")
    if not path.exists() or path.read_bytes() != encoded:
        path.write_bytes(encoded)


def _stylesheet(manifest: dict) -> str:
    parts = ["/* GENERATED from the authored `organization/theme/` CSS.\n * Do not edit by hand. */\n"]
    for name in ("typography", "base", "light", "dark"):
        file = f"theme/{name}.css"
        source = (BRAND / file).read_text(encoding="utf-8")
        source = re.sub(r"/\*.*?\*/|url\(\s*([\"']?)(.*?)\1\s*\)", _inline_asset, source, flags=re.DOTALL)
        parts.extend((f"\n/* BEGIN authored {file} */\n", source.rstrip("\n"), f"\n/* END authored {file} */\n"))
        if name != "base":
            continue
        parts.append("\n/* BEGIN generated corner radii from organization/organization.toml */\n:root {\n")
        radii = manifest.get("theme", {}).get("corner_radii", {})
        for key in ("base", "sm", "md", "lg", "xl", "2xl", "3xl", "4xl"):
            if key in radii:
                suffix = "" if key == "base" else f"-{key}"
                parts.append(f"  --brand-radius{suffix}: {radii[key]};\n")
        parts.append("}\n/* END generated corner radii */\n")
    return "".join(parts)


def _inline_asset(match: re.Match[str]) -> str:
    if match.group(0).startswith("/*"):
        return match.group(0)
    relative = match.group(2)
    if relative.startswith("data:"):
        return match.group(0)
    path = (BRAND / relative).resolve()
    if not path.is_relative_to(BRAND.resolve()):
        raise ValueError(f"organization asset escapes its pack: {relative}")
    mime = {".woff2": "font/woff2", ".svg": "image/svg+xml", ".png": "image/png"}.get(path.suffix)
    if mime is None:
        raise ValueError(f"unsupported organization asset: {relative}")
    data = base64.b64encode(path.read_bytes()).decode("ascii")
    return f'url("data:{mime};base64,{data}")'


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
