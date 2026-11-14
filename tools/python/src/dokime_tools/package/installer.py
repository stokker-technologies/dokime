# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Package Verified Desktop Product Bytes With Native Tauri Bundlers."""

# === Imports === #

# Standard Library Imports
import argparse
import hashlib
import json
import os
import subprocess
import sys
from pathlib import Path

# Local Imports
from dokime_tools import temporary
from dokime_tools.build import configuration, paths
from dokime_tools.package.staging import stage_file
from dokime_tools.process import spawn_flags

# === Public Functions === #


def bundle(root: Path, profile: str) -> list[Path]:
    """Consume Matching Product Bytes Without Compiling or Patching Them."""
    build = paths.output(root, "build", profile)
    source = build / "product/distribution"
    manifest = json.loads((source / "distribution.json").read_text(encoding="utf-8"))
    windows = sys.platform == "win32"
    suffix = ".exe" if windows else ""
    binary = source / "bin" / f"dokime-gui{suffix}"
    expected = manifest["files"][f"bin/dokime-gui{suffix}"]
    if hashlib.sha256(binary.read_bytes()).hexdigest() != expected:
        raise ValueError("Package Desktop: Product Executable Changed After Export")
    native = manifest["target"]
    if ("windows" in native) != windows:
        raise ValueError("Package Desktop: Use the Product Target's Native Bundler")
    config = json.loads((root / "crates/dokime-gui/src-tauri/tauri.conf.json").read_text(encoding="utf-8"))
    if not config["plugins"]["updater"]["pubkey"]:
        raise ValueError("Package Desktop: Compile the Updater Verification Key")
    signed = windows and bool(os.environ.get("TAURI_SIGNING_PRIVATE_KEY"))
    stable = os.environ.get("DOKIME_RELEASE_CONTEXT") == "true" or os.environ.get("GITHUB_REF", "").startswith("refs/tags/v")
    if windows and stable and not signed:
        raise ValueError("Package Desktop: Stable Publication Requires the Updater Signing Key")
    destination = paths.output(root, "package", profile) / "rust"
    destination.mkdir(parents=True, exist_ok=True)
    outputs = []
    with temporary.directory() as scratch:
        target = scratch / "target"
        mode = "debug" if profile == "dev" else "release"
        staged = target / mode / binary.name
        stage_file(binary, staged)
        resources = {}
        for name, digest in manifest["files"].items():
            path = source / name
            if hashlib.sha256(path.read_bytes()).hexdigest() != digest:
                raise ValueError(f"Package Desktop: Distribution Input Changed: {name}")
            if name != f"bin/dokime-gui{suffix}":
                resources[str(path)] = name.removeprefix("bin/")
        overlay = scratch / "bundle.json"
        bundle_config: dict[str, object] = {"active": True, "createUpdaterArtifacts": signed, "resources": resources}
        if windows:
            windows_config: dict[str, object] = {"nsis": {"installMode": "both"}}
            if os.environ.get("DOKIME_AUTHENTICODE_PROVIDER", "none") != "none":
                windows_config["signCommand"] = {
                    "cmd": "pwsh",
                    "args": ["-NoProfile", "-File", str(root / "tools/powershell/authenticode.ps1"), "-Operation", "Sign", "-Path", "%1"],
                }
            bundle_config["windows"] = windows_config
        overlay.write_text(
            json.dumps(
                {
                    "bundle": bundle_config,
                }
            ),
            encoding="utf-8",
        )
        environment = dict(os.environ, CARGO_TARGET_DIR=str(target))
        command = [
            "node",
            str(root / "crates/dokime-gui/node_modules/@tauri-apps/cli/tauri.js"),
            "bundle",
            "--ci",
            "--no-binary-patching",
            "--features",
            "custom-protocol",
            "--bundles",
            "nsis" if windows else "deb,rpm",
            "--config",
            str(overlay),
        ]
        if profile == "dev":
            command.append("--debug")
        result = subprocess.run(
            command, cwd=root / "crates/dokime-gui", env=environment, capture_output=True, text=True, check=False, creationflags=spawn_flags()
        )
        print(result.stdout, end="", flush=True)
        print(result.stderr, end="", file=sys.stderr, flush=True)
        if result.returncode:
            raise RuntimeError(f"Package Desktop: Native Bundler Failed ({result.returncode}): {result.stderr.strip()}")
        if hashlib.sha256(staged.read_bytes()).hexdigest() != expected:
            raise ValueError("Package Desktop: Bundler Modified the Product Executable")
        for artifact in sorted((target / mode / "bundle").rglob("*")):
            if not artifact.is_file() or not artifact.name.endswith((".exe", ".deb", ".rpm", ".sig")):
                continue
            extension = ".exe" if artifact.name.endswith(".exe") else ".exe.sig" if artifact.name.endswith(".exe.sig") else artifact.suffix
            ending = "-setup" + extension if windows else extension
            name = f"dokime-{manifest['version']}-{native}{ending}"
            stage_file(artifact, destination / name)
            outputs.append(destination / name)
        if not outputs:
            raise ValueError("Package Desktop: Native Bundler Produced No Installers")
        if signed:
            installer = next(path for path in outputs if path.suffix == ".exe")
            signature = installer.with_suffix(".exe.sig")
            if not signature.is_file():
                raise ValueError("Package Desktop: Signed Installer Has No Updater Signature")
            base = os.environ.get(
                "TAURI_UPDATER_ARTIFACT_BASE_URL",
                f"https://github.com/stokker-technologies/dokime/releases/download/v{manifest['version']}",
            ).rstrip("/")
            update = {
                "version": manifest["version"],
                "platforms": {
                    "windows-x86_64": {"signature": signature.read_text(encoding="utf-8").strip(), "url": base + "/" + installer.name}
                },
            }
            (destination / "latest.json").write_text(json.dumps(update, indent=2) + "\n", encoding="utf-8")
    return outputs


# === Entry Point === #

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--profile", choices=configuration.PROFILES, default="dev")
    bundle(configuration.ROOT, parser.parse_args().profile)
