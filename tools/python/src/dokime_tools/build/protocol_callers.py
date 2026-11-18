# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Check Protocol Caller Contracts"""

# === Imports === #

# Standard Library Imports
import ast
import re
import shutil
import subprocess
import tempfile
from pathlib import Path

# Third Party Imports
from google.protobuf.descriptor_pb2 import FileDescriptorSet
from harmos_tools.source.protocol import caller_options

# Local Imports
from ..process import spawn_flags

# Type Checking Imports


# === Public Exports === #

# === Constants === #

PYTHON_PUBLIC_HOMES = {
    "": frozenset(
        [
            "Artifact",
            "Case",
            "Cases",
            "Client",
            "Cycle",
            "Cycles",
            "Execution",
            "Executions",
            "Failure",
            "Note",
            "Outcome",
            "Plugin",
            "Plugins",
            "Progress",
            "PromptAnswer",
            "PromptQuestion",
            "RegisteredPlugin",
            "ResultStatus",
            "Scalar",
            "SdkError",
            "Suite",
            "connect",
        ]
    ),
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


def main() -> None:
    root = Path(__file__).resolve().parents[5]
    check_public_surfaces(root)

    scratch = Path(tempfile.mkdtemp())
    try:
        output = scratch / "descriptors.bin"
        command = ["buf", "build", "--as-file-descriptor-set", "-o", str(output)]
        subprocess.run(command, check=True, creationflags=spawn_flags())
        descriptors = FileDescriptorSet.FromString(output.read_bytes())
        methods = caller_options(descriptors)
        print(f"Protocol Callers: {len(methods)} Annotated Operations")
    finally:
        shutil.rmtree(scratch)


def check_public_surfaces(root: Path) -> None:
    """Keep Protocol Implementation Names Out of Consumer Entry Points.

    Allow internal generated directories (detail, _generated, _wire), canonical
    gRPC method paths. The separate ci:docs task checks guides.
    """
    wire = re.compile(r"\b_wire\b|\w+_wire\b|\b(?:Harmos|Dokime)\.Internal\b|\b(?:[A-Za-z_]\w*(?:\.|::))+v\d+\b|[/\\]v\d+[/\\]")
    grpc_path = re.compile(r"/?[a-zA-Z_]\w*(?:\.\w+)*\.v\d+(?:\.\w+)*/[A-Za-z_]\w*")
    candidates = sorted((root / "sdk/cpp/include").rglob("*.hpp"))
    candidates += sorted((root / "sdk/cpp/examples").glob("*/generated/*.hpp"))
    for path in (root / "sdk/python/src/dokime_sdk").rglob("*"):
        if path.suffix not in {".py", ".pyi"}:
            continue
        if any(part.startswith("_") for part in path.relative_to(root / "sdk/python/src/dokime_sdk").parts[:-1]):
            continue
        if path.name.startswith("_") and path.name != "__init__.py":
            continue
        candidates.append(path)
    findings = []
    for path in candidates:
        relative = path.relative_to(root)
        if any(part in {"detail", "_generated", "_wire"} for part in relative.parts):
            continue
        for number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
            line = grpc_path.sub("", line)
            if wire.search(line) or re.search(r"\bv\d+\b", line):
                findings.append(f"{relative.as_posix()}:{number}: Public Surface Names a Wire Package")
    if (root / "sdk/python/src/dokime").exists():
        findings.append("sdk/python/src/dokime: Remove the Retired Python Package; Use dokime_sdk")
    for path in sorted((root / "sdk/python/src/dokime_sdk").rglob("*")):
        if path.suffix not in {".py", ".pyi"}:
            continue
        for node in ast.walk(ast.parse(path.read_text(encoding="utf-8"))):
            if not isinstance(node, (ast.Import, ast.ImportFrom)):
                continue
            imports = []
            if isinstance(node, ast.Import):
                imports = [item.name for item in node.names]
            if isinstance(node, ast.ImportFrom) and not node.level and node.module:
                imports = [node.module]
            if any(name.split(".")[0] == "dokime" for name in imports):
                findings.append(f"{path.relative_to(root)}:{node.lineno}: Replace the Retired Python Import with dokime_sdk")
    package = root / "sdk/python/src/dokime_sdk"
    for module, expected in PYTHON_PUBLIC_HOMES.items():
        relative = f"{module}/__init__.py" if module == "templates" else f"{module}.py" if module else "__init__.py"
        path = package / relative
        if not path.is_file():
            if package.is_dir():
                findings.append(f"{relative}: Missing Python Public Home")
            continue
        tree = ast.parse(path.read_text(encoding="utf-8"))
        names = set()
        for node in tree.body:
            if isinstance(node, (ast.ClassDef, ast.FunctionDef, ast.AsyncFunctionDef)):
                names.add(node.name)
            if isinstance(node, ast.TypeAlias) and isinstance(node.name, ast.Name):
                names.add(node.name.id)
            if isinstance(node, (ast.Assign, ast.AnnAssign)):
                targets = node.targets if isinstance(node, ast.Assign) else [node.target]
                names.update(target.id for target in targets if isinstance(target, ast.Name))
            if isinstance(node, ast.ImportFrom):
                names.update(item.name for item in node.names if item.name in expected or (not module and not item.name.startswith("_")))
            if isinstance(node, (ast.Import, ast.ImportFrom)) and any(item.asname is not None for item in node.names):
                findings.append(f"{relative}:{node.lineno}: Remove Python Import Aliases")
        if "__getattr__" in names or "__dir__" in names:
            findings.append(f"{relative}: Remove Dynamic Python Public Exports")
        if "__all__" in names:
            findings.append(f"{relative}: Remove __all__; Declare the Three Public Homes Directly")
        names = {name for name in names if not name.startswith("_")}
        if names != expected:
            findings.append(
                f"{relative}: Python Public Inventory Missing {sorted(expected - names)}; Unexpected {sorted(names - expected)}"
            )
    for path in package.glob("*"):
        if path.name.startswith("_") or path.name in {"operations.py", "templates", "py.typed", "LICENSE", "NOTICE"}:
            continue
        if path.suffix in {".py", ".pyi"} or (path.is_dir() and any(item.suffix in {".py", ".pyi"} for item in path.rglob("*"))):
            findings.append(f"{path.relative_to(root)}: Remove the Fourth Python Public Home")
    for path in sorted((root / "sdk/csharp/src/Dokime").rglob("*.cs")):
        for number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
            if re.search(r"^namespace .*\.V\d+\b", line):
                findings.append(f"{path.relative_to(root)}:{number}: Public Wire Namespace")
    for path in sorted((root / "sdk/rust/src").rglob("*.rs")):
        for number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
            if re.search(r"\bpub\s+mod\s+(?:wire|v\d+)\b", line):
                findings.append(f"{path.relative_to(root).as_posix()}:{number}: Public Wire Module")
    if findings:
        raise ValueError(
            "Check Public Surfaces: Update Public SDK Exports to Hide Wire Names and Remove Duplicate Exports\n" + "\n".join(findings)
        )
    print("Public Surfaces: Headers, Stubs and Exports Passed")


# === Private Functions === #


# === Entry Point === #

if __name__ == "__main__":
    main()
