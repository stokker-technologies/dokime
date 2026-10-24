# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Compile The Generated Starter During Tests Build"""

# === Imports === #

# Standard Library Imports
import json
import os
import shutil
import subprocess
import sys
import tomllib
from pathlib import Path

# Third Party Imports
# Local Imports
from dokime_tools import temporary
from dokime_tools.build import paths

# Type Checking Imports

# === Public Exports === #

# === Constants === #

REPOSITORY = "https://github.com/stokker-technologies/dokime"

# === Types === #

# === Classes === #

# === Public Functions === #


def check(root: Path, profile: str, dependencies: Path) -> None:
    """Compile A Public Starter Offline Against This Build's Local SDK And Locked Inputs."""
    built = paths.output(root, "build", profile)
    executable = built / "bin" / ("cargo-dokime.exe" if os.name == "nt" else "cargo-dokime")
    environment = {
        key: value
        for key, value in os.environ.items()
        if not key.startswith("NEXTEST")
        and key
        not in (
            "VIRTUAL_ENV",
            "MISE_LOCKED",
            "MISE_ENABLE_TOOLS",
            "CARGO_TARGET_DIR",
            "UV_PROJECT_ENVIRONMENT",
            "UV_PROJECT",
            "UV_WORKING_DIR",
            "GIT_DIR",
            "GIT_WORK_TREE",
            "GIT_COMMON_DIR",
        )
    }

    with temporary.directory() as scratch:
        # Standalone Recipes Ignore Machine And Ancestor Project Tool Layers.
        configuration = scratch / "empty-mise.toml"
        configuration.touch()
        environment.update(
            MISE_GLOBAL_CONFIG_FILE=str(configuration),
            MISE_SYSTEM_CONFIG_FILE=str(configuration),
            MISE_CEILING_PATHS=_mise_path(scratch),
        )
        # Generate Through The Built Product In An Independent Git And Cargo Boundary.
        plugin = scratch / "starter-plugin"
        subprocess.run(
            [str(executable), "dokime", "new", str(plugin)],
            cwd=scratch,
            env=environment,
            check=True,
            creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
            stdin=sys.stdin,
            stdout=sys.stdout,
            stderr=sys.stderr,
        )
        subprocess.run(
            ["git", "init", "--initial-branch=feat/starter"],
            cwd=plugin,
            env=environment,
            check=True,
            creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
            stdin=sys.stdin,
            stdout=sys.stdout,
            stderr=sys.stderr,
        )

        # The Candidate Supplies Its Authoring Tool Before That Version Is Published.
        candidate = json.dumps(str(built))
        tool = f'"github:stokker-technologies/dokime" = {{ path = {candidate} }}\n'
        overrides = f'[tasks.build.tools]\n{tool}\n[tasks."package:rust".tools]\n{tool}'
        (plugin / "mise.local.toml").write_text(overrides, encoding="utf-8", newline="\n")

        # Replace The Published SDK Reference Only In This Disposable Consumer.
        manifest = plugin / "Cargo.toml"
        authored = manifest.read_text(encoding="utf-8")
        dependency = tomllib.loads(authored)["dependencies"]["dokime"]
        if dependency.get("git") != REPOSITORY or dependency.get("branch") != "dev":
            raise ValueError("the generated starter must declare the published Dokime SDK")
        lines = authored.splitlines(keepends=True)
        selected = [index for index, line in enumerate(lines) if line.startswith("dokime = ")]
        if len(selected) != 1:
            raise ValueError("the generated starter must have one SDK dependency declaration")
        local = json.dumps(str(root / "crates/dokime"))
        lines[selected[0]] = f'dokime = {{ path = {local}, features = ["plugin-sidecar"] }}\n'
        manifest.write_text("".join(lines), encoding="utf-8", newline="\n")

        # Resolve Every Cargo Source Against The Exported Vendor Tree, With An Empty Cargo Home.
        sources = tomllib.loads((dependencies / "cargo-config.toml").read_text(encoding="utf-8"))["source"]
        sections = []
        redirected = False
        for name, settings in sources.items():
            if "directory" in settings:
                settings["directory"] = str(dependencies / "vendor")
                redirected = True
            section = [f"[source.{json.dumps(name)}]"]
            section.extend(f"{key} = {json.dumps(value)}" for key, value in settings.items())
            sections.append("\n".join(section))
        if not redirected:
            raise ValueError("the exported Cargo configuration must name its vendor directory")
        (plugin / ".cargo").mkdir()
        (plugin / ".cargo/config.toml").write_text("\n\n".join(sections) + "\n", encoding="utf-8", newline="\n")
        shutil.copyfile(root / "Cargo.lock", plugin / "Cargo.lock")

        # Rebind Python Sources To The Staged Wheels Without Moving Any Shipped Pin.
        (plugin / "uv.toml").write_text("no-index = true\n", encoding="utf-8", newline="\n")
        environment.update(
            MISE_TRUSTED_CONFIG_PATHS=str(plugin),
            MISE_YES="1",
            CARGO_NET_OFFLINE="true",
            CARGO_HOME=str(plugin / "target/cargo-home"),
            UV_OFFLINE="true",
            UV_CONFIG_FILE=str(plugin / "uv.toml"),
            UV_FIND_LINKS=str(dependencies / "wheels"),
            UV_CACHE_DIR=str(plugin / "target/uv-cache"),
        )
        # Reconcile The Disposable Workspace Identity After Its Local SDK Override.
        subprocess.run(
            ["mise", "exec", "--", "cargo", "update", "--offline", "--workspace"],
            cwd=plugin,
            env=environment,
            check=True,
            creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
            stdin=sys.stdin,
            stdout=sys.stdout,
            stderr=sys.stderr,
        )
        lock = plugin / "uv.lock"
        before = _pins(lock)
        subprocess.run(
            ["mise", "exec", "--", "uv", "lock", "--offline", "--project", "tools/python"],
            cwd=plugin,
            env=environment,
            check=True,
            creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
            stdin=sys.stdin,
            stdout=sys.stdout,
            stderr=sys.stderr,
        )
        if _pins(lock) != before:
            raise ValueError("the offline wheelhouse changed the starter's shipped Python pins")

        # The Local SDK Path Varies With The Checkout, So Its Override Takes The Starter's Own TOML Layout.
        subprocess.run(
            ["mise", "run", "format:toml"],
            cwd=plugin,
            env=environment,
            check=True,
            creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
            stdin=sys.stdin,
            stdout=sys.stdout,
            stderr=sys.stderr,
        )

        # The Starter's Own Recipes Prove Its Feature Edges, Product, Test Archive And Unit Lane.
        subprocess.run(
            ["mise", "run", "check"],
            cwd=plugin,
            env=environment,
            check=True,
            creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
            stdin=sys.stdin,
            stdout=sys.stdout,
            stderr=sys.stderr,
        )
        for task in ("build", "build:rust:tests", "test:rust:unit"):
            subprocess.run(
                ["mise", "run", task, "--profile", profile],
                cwd=plugin,
                env=environment,
                check=True,
                creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
                stdin=sys.stdin,
                stdout=sys.stdout,
                stderr=sys.stderr,
            )


# === Private Functions === #


def _mise_path(path: Path) -> str:
    value = str(path)
    if os.name == "nt" and value.startswith("\\\\?\\"):
        value = value[4:]
        if value.startswith("UNC\\"):
            value = "\\\\" + value[4:]
    return value


def _pins(lock: Path) -> list[tuple[str, str]]:
    packages = tomllib.loads(lock.read_text(encoding="utf-8"))["package"]
    return sorted((package["name"], package["version"]) for package in packages)


# === Entry Point === #
