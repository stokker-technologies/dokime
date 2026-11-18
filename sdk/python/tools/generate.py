# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Build Python Bindings from the Authoritative Protocol Sources."""

# === Imports === #

# Standard Library Imports

import argparse
import ast
import hashlib
import json
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

# Third Party Imports
import grpc_tools
from grpc_tools import protoc

# Local Imports

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def main(*, check: bool = False, exports_only: bool = False) -> None:
    sdk = Path(__file__).resolve().parents[1]
    root = sdk.parents[1]
    bundled_protocols = sdk / "proto"
    protocols = bundled_protocols if bundled_protocols.is_dir() else root / "proto"
    source = sdk / "src/dokime"
    output = Path(tempfile.mkdtemp(prefix="dokime-protocol-"))
    try:
        if not exports_only:
            arguments = [
                "protoc",
                f"-I{protocols}",
                f"-I{Path(grpc_tools.__file__).parent / '_proto'}",
                f"--python_out={output}",
                f"--pyi_out={output}",
                f"--grpc_python_out={output}",
                f"--descriptor_set_out={output / 'surface.pb'}",
                "--include_imports",
                *map(str, sorted(protocols.glob("dokime/**/*.proto"))),
            ]
            status = protoc.main(arguments)
            if status:
                raise RuntimeError("Generate Python Protocols: Correct the Reported Proto Errors and Regenerate")
            if not check:
                for old in source.rglob("*_pb2*"):
                    if old.is_file():
                        old.unlink()
            drift: list[str] = []
            expected: set[Path] = set()
            for path in (output / "dokime").rglob("*"):
                if not path.is_file():
                    continue
                target = source / "_wire" / path.relative_to(output / "dokime")
                expected.add(target)
                if not check:
                    target.parent.mkdir(parents=True, exist_ok=True)
                text = path.read_text(encoding="utf-8")
                for owner in ("dokime", "harmos"):
                    for executor in ("model", "host", "plugin"):
                        text = text.replace(f"from {owner}.{executor}", f"from {owner}._wire.{executor}")
                        text = text.replace(f"import {owner}.{executor}.", f"import {owner}._wire.{executor}.")
                        text = text.replace(
                            f"BuildTopDescriptorsAndMessages(DESCRIPTOR, '{owner}.{executor}.",
                            f"BuildTopDescriptorsAndMessages(DESCRIPTOR, '{owner}._wire.{executor}.",
                        )
                if check:
                    if not target.is_file() or target.read_text(encoding="utf-8") != text:
                        drift.append(str(target.relative_to(sdk)))
                else:
                    target.write_text(text, encoding="utf-8", newline="\n")
                    (target.parent / "py.typed").touch()
                    (target.parent / "__init__.py").touch(exist_ok=True)
            if check:
                existing = {path for path in (source / "_wire").rglob("*_pb2*") if path.is_file() and path.suffix in {".py", ".pyi"}}
                drift.extend(str(path.relative_to(sdk)) for path in sorted(existing - expected))
                if (output / "surface.pb").read_bytes() != (source / "_wire/surface.pb").read_bytes():
                    drift.append("_wire/surface.pb")
            else:
                shutil.copyfile(output / "surface.pb", source / "_wire/surface.pb")
            manifest = {}
            for path in protocols.rglob("*.proto"):
                relative = path.relative_to(protocols).as_posix()
                digest = hashlib.sha256(path.read_bytes()).hexdigest()
                manifest[relative] = digest
            text = json.dumps(manifest, sort_keys=True) + "\n"
            if check:
                if (source / "_wire/sources.json").read_text(encoding="utf-8") != text:
                    drift.append("_wire/sources.json")
                if drift:
                    raise ValueError("Check Python Generation: Regenerate With mise run generate -- --protocol: " + ", ".join(drift))
            else:
                (source / "_wire/sources.json").write_text(text, encoding="utf-8", newline="\n")
        # Compose the Product Authoring Barrel From the Generated Model Export Inventory.
        shared = ast.parse((source / "_generated/shared.py").read_text(encoding="utf-8"))
        names = sorted(
            {
                item.name
                for node in shared.body
                if isinstance(node, ast.ImportFrom)
                for item in node.names
                if item.name != "Visibility" and not item.name.startswith("VISIBILITY_")
            }
        )
        additions = {
            "._context": ["Context"],
            "._plugin": ["Plugin"],
            ".authoring": [
                "Case",
                "Template",
                "criterion",
                "materialize",
                "observer",
                "prompt",
                "record",
                "step",
                "stream",
                "suite",
                "window_observer",
            ],
            ".operations": ["Operation", "Visibility", "operation"],
            "harmos.plugin": ["Window"],
            "harmos": ["FailureValue"],
        }
        lines = [
            "# @generated from protocol descriptors and product authoring exports; do not edit.",
            "from harmos._invocation import FailureError",
        ]
        lines.extend(f"from ._generated.shared import {name}" for name in names)
        for module, exports in additions.items():
            lines.extend(f"from {module} import {name}" for name in exports)
        exports = sorted({"FailureError", *names, *(name for values in additions.values() for name in values)})
        lines.extend(["", "__all__ = " + repr(exports), ""])
        barrel = output / "__init__.py"
        barrel.write_text("\n".join(lines), encoding="utf-8", newline="\n")
        subprocess.run([sys.executable, "-m", "ruff", "check", "--config", str(sdk / "ruff.toml"), "--select", "I,RUF022", "--fix", str(barrel)], check=True)
        subprocess.run([sys.executable, "-m", "ruff", "format", "--config", str(sdk / "ruff.toml"), str(barrel)], check=True)
        text = barrel.read_text(encoding="utf-8")
        target = source / "__init__.py"
        if check:
            if target.read_text(encoding="utf-8") != text:
                raise ValueError("Check Python Exports: Run mise -C sdk/python run generate")
        else:
            target.write_text(text, encoding="utf-8", newline="\n")
    finally:
        shutil.rmtree(output)


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--exports-only", action="store_true")
    options = parser.parse_args()
    main(check=options.check, exports_only=options.exports_only)
