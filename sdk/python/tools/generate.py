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
    "_values": ["Artifact", "CatalogTemplate", "Note", "Outcome", "Progress", "PromptAnswer", "PromptQuestion", "ResultStatus", "Scalar"],
}

PUBLIC_HOMES = {
    "": frozenset(name for names in ROOT_EXPORTS.values() for name in names),
    "operations": frozenset(
        [
            "Context",
            "OperationError",
            "OperationPolicy",
            "Payload",
            "RetryPolicy",
            "Resource",
            "Secret",
            "Setting",
            "Settings",
            "StreamSubscription",
            "Visibility",
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
            "FilterOperator",
            "FtpLocation",
            "GridAxisEntry",
            "GridHighlight",
            "LineField",
            "LineMarker",
            "LineMatcher",
            "Materialization",
            "MirrorCapture",
            "Parameter",
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
            "Observer",
            "pie",
            "plot_card",
            "prompt",
            "record",
            "records_card",
            "step",
            "stream",
            "time_series",
            "trajectory",
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


def main(*, check: bool = False) -> None:
    """Write or Check the Installed Surface Record and the Closed Product Root; the Library CLI Owns the Wire Modules."""
    import grpc_tools
    from grpc_tools import protoc

    sdk = Path(__file__).resolve().parents[1]
    root = sdk.parents[1]
    bundled_protocols = sdk / "proto"
    protocols = bundled_protocols if bundled_protocols.is_dir() else root / "target/protocol"
    source = sdk / "src/dokime_sdk"
    output = Path(tempfile.mkdtemp(prefix="dokime-protocol-"))
    try:
        # The Installed Surface Records Its Descriptors and Sources, So the Wheel Checks Can Prove It Current.
        manifest = source_inventory(protocols)
        arguments = [
            "protoc",
            f"-I{protocols}",
            f"-I{Path(grpc_tools.__file__).parent / '_proto'}",
            f"--descriptor_set_out={output / 'surface.pb'}",
            "--include_imports",
            *(str(protocols / name) for name in manifest if name.startswith("dokime/")),
        ]
        if protoc.main(arguments):
            raise RuntimeError("Generate Python Surface: Correct the Reported Proto Errors and Regenerate")
        (output / "sources.json").write_text(json.dumps(manifest, sort_keys=True) + "\n", encoding="utf-8", newline="\n")
        # Generate the Closed Product Root; Operations and Templates Own Their Contexts.
        lines = ["# @generated from the product public-home inventory; do not edit."]
        lines.extend(f"from . import {module}" for module in ROOT_EXPORTS)
        lines.append("")
        for module, names in ROOT_EXPORTS.items():
            lines.extend(f"{name} = {module}.{name}" for name in names)
        barrel = output / "__init__.py"
        barrel.write_text("\n".join(lines) + "\n", encoding="utf-8", newline="\n")
        subprocess.run(
            [sys.executable, "-m", "ruff", "check", "--config", str(sdk / "ruff.toml"), "--select", "I", "--fix", str(barrel)], check=True
        )
        subprocess.run([sys.executable, "-m", "ruff", "format", "--config", str(sdk / "ruff.toml"), str(barrel)], check=True)
        targets = {
            output / "surface.pb": source / "_generated/surface.pb",
            output / "sources.json": source / "_generated/sources.json",
            barrel: source / "__init__.py",
        }
        drift = [
            str(target.relative_to(sdk))
            for rendered, target in targets.items()
            if not target.is_file() or target.read_bytes() != rendered.read_bytes()
        ]
        if check and drift:
            raise ValueError("Check Python Surface: Regenerate With mise run generate -- --protocol --language python: " + ", ".join(drift))
        if not check:
            for rendered, target in targets.items():
                shutil.copyfile(rendered, target)
    finally:
        shutil.rmtree(output)


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    options = parser.parse_args()
    main(check=options.check)
