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
import hashlib
import json
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

# Third Party Imports

# Local Imports

# Type Checking Imports

# === Public Exports === #

# === Constants === #

ROOT_EXPORTS = {
    "_authoring": ["Suite"],
    "_plugin": ["Lane", "Plugin"],
    "_client": ["Case", "Cases", "Client", "Cycle", "Cycles", "Execution", "Executions", "Plugins", "RegisteredPlugin"],
    "_connection": ["connect"],
    "_context": ["StreamRow", "Gap"],
    "_errors": ["Failure", "SdkError"],
    "_values": ["Artifact", "Note", "Outcome", "Progress", "PromptAnswer", "PromptQuestion", "ResultStatus", "Scalar"],
}

PUBLIC_HOMES = {
    "": frozenset(name for names in ROOT_EXPORTS.values() for name in names),
    # SDK Surface Specification S3 Defers SettingGroup Until Its Metadata Contract Is Agreed.
    "operations": frozenset(
        [
            "BoundOperation",
            "Context",
            "FailureError",
            "Operation",
            "OperationPolicy",
            "Payload",
            "RetryPolicy",
            "SecretDeclaration",
            "SettingDeclaration",
            "SettingValue",
            "StreamSubscription",
            "Visibility",
            "declaration",
            "operation",
        ]
    ),
    "templates": frozenset(
        [
            "Aggregate",
            "AxisScale",
            "BarArrangement",
            "CaptureGap",
            "CaptureMode",
            "CaptureResult",
            "CapturedArtifact",
            "Card",
            "Context",
            "ContourRender",
            "DataExpression",
            "Document",
            "Environment",
            "Expression",
            "FileLocation",
            "FileParameter",
            "FileRule",
            "FileRuleAction",
            "FileSelection",
            "FileSource",
            "FtpLocation",
            "GridAxisEntry",
            "GridHighlight",
            "LineField",
            "LineMarker",
            "LineMatcher",
            "Materialization",
            "MirrorCapture",
            "Parameter",
            "ParameterDefinition",
            "Plot",
            "PlotStyle",
            "Prompt",
            "PromptDeclaration",
            "Record",
            "RecordRow",
            "RecordsQuery",
            "Reduction",
            "ReferenceBand",
            "ReferenceLine",
            "SampleRow",
            "SamplesQuery",
            "Section",
            "Severity",
            "Statistics",
            "Stream",
            "StreamCapture",
            "Summary",
            "Termination",
            "TestTemplate",
            "Window",
            "XyInterval",
            "bar",
            "box_plot",
            "contour",
            "criterion",
            "field",
            "frequency_series",
            "grid_chart",
            "heat_strip",
            "histogram",
            "materialize",
            "observer",
            "pie",
            "plot_card",
            "prompt",
            "record",
            "records_card",
            "step",
            "stream",
            "time_series",
            "trajectory",
            "window_observer",
            "xy",
        ]
    ),
}

# === Types === #

# === Classes === #

# === Public Functions === #


def source_inventory(protocols: Path) -> dict[str, str]:
    """Fingerprint Every Authoritative Proto for Generation and Distribution Checks."""
    return {
        path.relative_to(protocols).as_posix(): hashlib.sha256(path.read_bytes()).hexdigest() for path in sorted(protocols.rglob("*.proto"))
    }


def main(*, check: bool = False, exports_only: bool = False) -> None:
    sdk = Path(__file__).resolve().parents[1]
    root = sdk.parents[1]
    bundled_protocols = sdk / "proto"
    protocols = bundled_protocols if bundled_protocols.is_dir() else root / "target/protocol"
    python_package = "dokime_sdk"
    source = sdk / "src" / python_package
    output = Path(tempfile.mkdtemp(prefix="dokime-protocol-"))
    try:
        if not exports_only:
            import grpc_tools
            from grpc_tools import protoc

            manifest = source_inventory(protocols)
            arguments = [
                "protoc",
                f"-I{protocols}",
                f"-I{Path(grpc_tools.__file__).parent / '_proto'}",
                f"--python_out={output}",
                f"--pyi_out={output}",
                f"--grpc_python_out={output}",
                f"--descriptor_set_out={output / 'surface.pb'}",
                "--include_imports",
                *(str(protocols / name) for name in manifest if name.startswith("dokime/")),
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
                if path.suffix == ".pyi":
                    # Fields Are Managed by the Protobuf Metaclass, Not Python Slots.
                    text = ("from google.protobuf.reflection import GeneratedProtocolMessageType\nfrom typing import final\n\n") + text
                    text = re.sub(
                        r"(?m)^( *)class (\w+)\(_message.Message\):\n +__slots__ = [^\n]+\n",
                        r"\1@final\n\1class \2(_message.Message, metaclass=GeneratedProtocolMessageType):\n\1    __slots__: tuple[str, ...]\n",
                        text,
                    )
                for owner in ("dokime", "harmos"):
                    package = python_package if owner == "dokime" else owner
                    for executor in ("model", "host", "plugin"):
                        text = text.replace(f"from {owner}.{executor}", f"from {package}._wire.{executor}")
                        text = text.replace(f"import {owner}.{executor}.", f"import {package}._wire.{executor}.")
                        text = text.replace(
                            f"BuildTopDescriptorsAndMessages(DESCRIPTOR, '{owner}.{executor}.",
                            f"BuildTopDescriptorsAndMessages(DESCRIPTOR, '{package}._wire.{executor}.",
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
            text = json.dumps(manifest, sort_keys=True) + "\n"
            if check:
                if (source / "_wire/sources.json").read_text(encoding="utf-8") != text:
                    drift.append("_wire/sources.json")
                if drift:
                    raise ValueError("Check Python Generation: Regenerate With mise run generate -- --protocol: " + ", ".join(drift))
            else:
                (source / "_wire/sources.json").write_text(text, encoding="utf-8", newline="\n")
        # Retire the Shared Generator's Public Transport Facade.
        facade = source / "client.py"
        if check and facade.exists():
            raise ValueError("Check Python Exports: Retire client.py With mise -C sdk/python run generate")
        if not check:
            facade.unlink(missing_ok=True)
        # Generate the Closed Product Root; Operations and Templates Own Their Contexts.
        lines = ["# @generated from the product public-home inventory; do not edit."]
        lines.extend(f"from . import {module}" for module in ROOT_EXPORTS)
        lines.append("")
        for module, names in ROOT_EXPORTS.items():
            lines.extend(f"{name} = {module}.{name}" for name in names)
        barrel = output / "__init__.py"
        barrel.write_text("\n".join(lines) + "\n", encoding="utf-8", newline="\n")
        subprocess.run(
            [sys.executable, "-m", "ruff", "check", "--config", str(sdk / "ruff.toml"), "--select", "I", "--fix", str(barrel)],
            check=True,
        )
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
