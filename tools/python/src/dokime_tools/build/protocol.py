# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Generate or Check Protocol Bindings and Composed SDK Roots"""

# === Imports === #

# Standard Library Imports
import argparse
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

# isort: split

# Third Party Imports
from harmos_tools.build.resources import sdk
from harmos_tools.build.resources.configuration import Configuration

# isort: split

# Local Imports
from dokime_tools.build import cargo
from dokime_tools.process import spawn_flags

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def configuration(root: Path) -> Configuration:
    """Declare Product Names, Compiler Inputs and Public Call Selectors."""
    return Configuration(
        namespace="dokime",
        python_package="dokime_sdk",
        csharp_namespace="Dokime",
        csharp_tools=root / "target/cs/packages/grpc.tools/2.76.0/tools",
        rustfmt=root / "rustfmt.toml",
        python_style=root / "sdk/python/ruff.toml",
        client_required_fields={"harmos.host.v1.plugins.SettingsListRequest": {"name"}},
        rust_selectors={
            "dokime.host.v1.DocumentsGetRequest": {"scope", "document_id"},
            "dokime.host.v1.DocumentsWatchRequest": {"scope", "document_id"},
            "dokime.host.v1.EvidenceStatisticsRequest": {"scope", "stream", "field"},
            "dokime.host.v1.evidence.RecordsListRequest": {"scope", "record_type"},
            "dokime.host.v1.evidence.RecordsWatchRequest": {"scope", "record_type"},
            "dokime.host.v1.evidence.SamplesListRequest": {"scope", "streams"},
            "dokime.host.v1.evidence.SamplesWatchRequest": {"scope", "streams"},
        },
    )


def generate(*, check: bool = False, root: Path, descriptors: Path, configuration: Configuration, language: str | None = None) -> None:
    """Publish Shared Projections Except the SDK-Owned Python Barrels."""
    output = Path(tempfile.mkdtemp(prefix="dokime-sdk-"))
    try:
        sdk.generate(root=output, descriptors=descriptors, configuration=configuration)
        barrels = {
            Path("sdk/python/src/dokime_sdk/__init__.py"),
            Path("sdk/python/src/dokime_sdk/client.py"),
            Path("sdk/python/src/dokime_sdk/plugin/__init__.py"),
        }
        expected = sdk.inventory(output, configuration) - barrels
        existing = sdk.inventory(root, configuration) - barrels
        if language == "python":
            expected = {path for path in expected if path.parts[:2] == ("sdk", "python")}
            existing = {path for path in existing if path.parts[:2] == ("sdk", "python")}
        if check:
            differences = [str(path) for path in sorted(expected ^ existing)]
            differences.extend(
                str(path) for path in sorted(expected & existing) if (root / path).read_bytes() != (output / path).read_bytes()
            )
            if differences:
                raise ValueError("Check SDK Definitions: Run mise run generate -- --protocol\n" + "\n".join(differences))
        else:
            for relative in sorted(existing - expected):
                (root / relative).unlink()
            for relative in sorted(expected):
                destination = root / relative
                destination.parent.mkdir(parents=True, exist_ok=True)
                destination.write_bytes((output / relative).read_bytes())
        command = [sys.executable, str(root / "sdk/python/tools/generate.py"), "--exports-only"]
        if check:
            command.append("--check")
        subprocess.run(
            command,
            check=True,
            creationflags=spawn_flags(),
            stdin=sys.stdin,
            stdout=sys.stdout,
            stderr=sys.stderr,
        )
    finally:
        shutil.rmtree(output)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--language", choices=("python",))
    options = parser.parse_args()
    if options.check and options.language is None:
        # Verify Protocol Output Against Its Generator.
        command = ["mise", "run", "test", "--", "--package", "dokime-protocol", "--target", "integration"]
        subprocess.run(
            command,
            check=True,
            creationflags=spawn_flags(),
            stdin=sys.stdin,
            stdout=sys.stdout,
            stderr=sys.stderr,
        )
    elif options.language is None:
        status = cargo.focused(Path.cwd(), "run", ["--locked", "--package", "dokime-protocol-generator"])
        if status:
            raise SystemExit(status)
    # Restore the Protocol Compiler Without Building the Library SDK.
    command = ["mise", "-C", "sdk/csharp", "run", "prepare:protocol"]
    subprocess.run(
        command,
        check=True,
        creationflags=spawn_flags(),
        stdin=sys.stdin,
        stdout=sys.stdout,
        stderr=sys.stderr,
    )
    root = Path.cwd()
    selected = configuration(root)
    generate(
        check=options.check,
        root=root,
        descriptors=root / "crates/dokime-protocol/src/descriptors.bin",
        configuration=selected,
        language=options.language,
    )
    # Generate or Verify Python Wire Modules Through Their SDK Owner.
    command = ["mise", "-C", "sdk/python", "run", "generate"]
    if options.check:
        command += ["--", "--check"]
    subprocess.run(
        command,
        check=True,
        creationflags=spawn_flags(),
        stdin=sys.stdin,
        stdout=sys.stdout,
        stderr=sys.stderr,
    )


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    main()
