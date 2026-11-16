# ***
#
# dokime-rust-workspace
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Compile the Generated SDK Surfaces Independently of Product Consumers."""

# === Imports === #

# Standard Library Imports
import argparse
import json
import subprocess
import sys
from pathlib import Path

# Third Party Imports
# Local Imports
from dokime_tools.process import spawn_flags

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--language", choices=("rust", "python", "csharp", "cpp", "typescript"))
    options = parser.parse_args()
    root = Path(__file__).resolve().parents[5]
    selected = [options.language] if options.language else ["rust", "python", "csharp", "cpp", "typescript"]
    for language in selected:
        if language == "rust":
            commands = [
                [
                    "mise",
                    "run",
                    "build",
                    "--",
                    "--cargo",
                    "check",
                    "--locked",
                    "--offline",
                    "--package",
                    "dokime",
                    "--test",
                    "generation",
                ]
            ]
        elif language == "python":
            # The Compiler Story Consumes the Wheel Installed by sdk/python's Install Task.
            commands = [
                [
                    "uv",
                    "run",
                    "--no-sync",
                    "--project",
                    "tools/python",
                    "python",
                    "-c",
                    "import runpy; runpy.run_path('sdk/python/tests/generation/test_surface.py')",
                ],
                [
                    "uv",
                    "run",
                    "--no-sync",
                    "--project",
                    "tools/python",
                    "ruff",
                    "check",
                    "sdk/python/src/dokime",
                    "sdk/python/tests/generation",
                ],
                [
                    "uv",
                    "run",
                    "--no-sync",
                    "--project",
                    "tools/python",
                    "ty",
                    "check",
                    "--project",
                    "sdk/python",
                    "sdk/python/src/dokime/__init__.py",
                    "sdk/python/src/dokime/client.py",
                    "sdk/python/src/dokime/plugin",
                    "sdk/python/src/dokime/_generated",
                    "sdk/python/tests/generation",
                ],
            ]
        elif language == "csharp":
            commands = [["mise", "-C", "sdk/csharp", "run", "test:generation"]]
        elif language == "cpp":
            commands = [["mise", "-C", "sdk/cpp", "run", "build", "--", "--preset", "tests", "--target", "dokime-generation"]]
        else:
            configuration = {
                "extends": str(root / "crates/dokime-gui/tsconfig.json"),
                "include": [],
                "files": [
                    str(root / "crates/dokime-gui/tests/generation/surface.ts"),
                    str(root / "crates/dokime-gui/src/types/app-version.d.ts"),
                ],
            }
            output = root / "target/generated/typescript-story.json"
            output.parent.mkdir(parents=True, exist_ok=True)
            output.write_text(json.dumps(configuration), encoding="utf-8")
            commands = [["pnpm", "--dir", "crates/dokime-gui", "exec", "tsc", "--project", str(output)]]

        # Compile the Selected Surface and Its Language Checks.
        for command in commands:
            subprocess.run(
                command,
                cwd=root,
                check=True,
                stdin=sys.stdin,
                stdout=sys.stdout,
                stderr=sys.stderr,
                creationflags=spawn_flags(),
            )
        print(f"Generation Story: {language} Passed", flush=True)


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    main()
