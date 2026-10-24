# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Prove The Generated Starter In The Nightly Lane"""

# === Imports === #

# Standard Library Imports
import json
import os
import shutil
import subprocess
import sys
import tomllib
from time import perf_counter

# Third Party Imports
# Local Imports
from dokime_tools import temporary
from dokime_tools.build import configuration, paths

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def test(profile: str) -> None:
    """Generate And Validate A Standalone Consumer Only In The Nightly Lane."""
    root = configuration.ROOT
    product = paths.output(root, "build", profile)
    tool = product / "bin" / f"cargo-dokime{'.exe' if os.name == 'nt' else ''}"
    if not tool.is_file():
        raise ValueError(f"missing candidate starter generator: {tool}")
    environment = {
        key: value
        for key, value in os.environ.items()
        if not key.startswith("NEXTEST")
        and key
        not in {
            "VIRTUAL_ENV",
            "MISE_LOCKED",
            "MISE_ENABLE_TOOLS",
            "MISE_TASK_SKIP",
            "CARGO_TARGET_DIR",
            "UV_PROJECT_ENVIRONMENT",
            "UV_PROJECT",
            "UV_WORKING_DIR",
            "GIT_DIR",
            "GIT_WORK_TREE",
            "GIT_COMMON_DIR",
        }
    }
    started = perf_counter()
    with temporary.directory() as scratch:
        neutral = scratch / "empty-mise.toml"
        neutral.touch()
        ceiling = str(scratch)
        if os.name == "nt" and ceiling.startswith("\\\\?\\"):
            ceiling = ceiling[4:]
            if ceiling.startswith("UNC\\"):
                ceiling = "\\\\" + ceiling[4:]
        environment.update(MISE_GLOBAL_CONFIG_FILE=str(neutral), MISE_SYSTEM_CONFIG_FILE=str(neutral), MISE_CEILING_PATHS=ceiling)
        plugin = scratch / "starter-plugin"
        for command, cwd in [
            ([str(tool), "dokime", "new", str(plugin)], scratch),
            (["git", "init", "--initial-branch=feat/starter"], plugin),
        ]:
            subprocess.run(
                command,
                cwd=cwd,
                env=environment,
                check=True,
                stdin=sys.stdin,
                stdout=sys.stdout,
                stderr=sys.stderr,
                creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
            )
        candidate = json.dumps(str(product))
        declaration = f'"github:stokker-technologies/dokime" = {{ path = {candidate} }}\n'
        (plugin / "mise.local.toml").write_text(
            f'[tasks.build.tools]\n{declaration}\n[tasks."package:rust".tools]\n{declaration}', encoding="utf-8", newline="\n"
        )

        # Only The Disposable Consumer Uses The Current SDK Before Publication.
        manifest = plugin / "Cargo.toml"
        authored = manifest.read_text(encoding="utf-8")
        dependency = tomllib.loads(authored)["dependencies"]["dokime"]
        if dependency.get("git") != "https://github.com/stokker-technologies/dokime" or dependency.get("branch") != "dev":
            raise ValueError("the starter must declare its public SDK dependency")
        lines = authored.splitlines(keepends=True)
        selected = [index for index, line in enumerate(lines) if line.startswith("dokime = ")]
        if len(selected) != 1:
            raise ValueError("the starter must declare one SDK dependency")
        lines[selected[0]] = f'dokime = {{ path = {json.dumps(str(root / "crates/dokime"))}, features = ["plugin-sidecar"] }}\n'
        manifest.write_text("".join(lines), encoding="utf-8", newline="\n")

        # Vendor The Locked Cargo Sources Without Compiling A Second Product.
        vendor = scratch / "vendor"
        vendor_config = scratch / "cargo-config.toml"
        with vendor_config.open("w", encoding="utf-8") as output:
            subprocess.run(
                ["cargo", "vendor", "--locked", "--versioned-dirs", str(vendor)],
                cwd=root,
                env=environment,
                stdout=output,
                stdin=sys.stdin,
                stderr=sys.stderr,
                check=True,
                creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
            )
        (plugin / ".cargo").mkdir()
        shutil.copyfile(vendor_config, plugin / ".cargo/config.toml")
        shutil.copyfile(root / "Cargo.lock", plugin / "Cargo.lock")
        environment.update(
            MISE_TRUSTED_CONFIG_PATHS=str(plugin),
            MISE_YES="1",
            CARGO_NET_OFFLINE="true",
            CARGO_HOME=str(scratch / "cargo-home"),
            CARGO_TARGET_DIR=str(root / "target"),
        )

        # uv Owns Its Normal Lock And Cache Integrity; No Custom Wheel Hash Gate.
        commands = [
            ["mise", "exec", "--", "cargo", "update", "--offline", "--workspace"],
            ["mise", "exec", "--", "uv", "sync", "--locked", "--project", "tools/python"],
            ["mise", "run", "format:toml"],
        ]
        for command in commands:
            subprocess.run(
                command,
                cwd=plugin,
                env=environment,
                check=True,
                stdin=sys.stdin,
                stdout=sys.stdout,
                stderr=sys.stderr,
                creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
            )
        environment["UV_OFFLINE"] = "true"
        print(f"[test:rust:starter] Prepared In {perf_counter() - started:.2f}s; running real standalone recipes", flush=True)
        for task in ("check", "build", "build:rust:tests", "test:rust:unit"):
            command = ["mise", "run", task]
            if task != "check":
                command += ["--profile", profile]
            subprocess.run(
                command,
                cwd=plugin,
                env=environment,
                check=True,
                stdin=sys.stdin,
                stdout=sys.stdout,
                stderr=sys.stderr,
                creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
            )
    print(f"[test:rust:starter] Completed In {perf_counter() - started:.2f}s", flush=True)


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    raise SystemExit(configuration.invoke(test))
