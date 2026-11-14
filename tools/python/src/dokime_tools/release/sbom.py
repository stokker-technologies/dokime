# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Generate Source and Artifact CycloneDX Inventories."""

# === Imports === #

# Standard Library Imports
import argparse
import hashlib
import json
import os
import shutil
import subprocess
import sys
import tarfile
import tempfile
import tomllib
import zipfile
from datetime import UTC, date, datetime
from pathlib import Path

# Third Party Imports
from harmos_tools.release import policy

# Local Imports
from ..process import spawn_flags

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def exceptions(directory: Path, *, today: date | None = None) -> dict:
    """Reject Expired or Incomplete Exceptions Before Any Tool Can Apply Them."""
    document = tomllib.loads((directory / "tools/config/supply-chain-exceptions.toml").read_text(encoding="utf-8"))
    today = datetime.now(UTC).date() if today is None else today
    for kind, entries in document.items():
        if kind not in {"advisory", "license", "ban"}:
            raise policy.ToolError(f"unknown exception category: {kind}")
        for entry in entries:
            for field in ("package", "version", "reason", "owner", "fixed"):
                if not entry.get(field):
                    raise policy.ToolError(f"exception is missing {field}")
            if kind == "ban":
                manifest = tomllib.loads(policy.inside(directory, entry["manifest"]).read_text(encoding="utf-8"))
                digest = hashlib.sha256(json.dumps(manifest, sort_keys=True).encode()).hexdigest()
                if digest != entry["manifest_sha256"]:
                    raise policy.ToolError(f"ban exception no longer matches {entry['package']}'s manifest")
            expiry = entry.get("expires")
            if not isinstance(expiry, date) or expiry <= today:
                raise policy.ToolError(f"expired exception: {entry['package']} {entry.get('id', entry.get('license'))}")
    return document


def accepted(document: dict, category: str, package: str, version: str, identifier: str) -> dict | None:
    """Scope Every Exception to One Finding and Exact Dependency Version."""
    for entry in document.get(category, []):
        if entry["package"] == package and entry["version"] == version and entry.get("id", entry.get("license")) == identifier:
            return entry
    return None


def check(directory: Path) -> int:
    """Keep Native Checks Strict; Accept Only Reviewed, Unexpired Findings."""
    document = exceptions(directory)
    output = directory / "target/supply"
    output.mkdir(parents=True, exist_ok=True)
    commands = [
        (
            "deny",
            [
                "cargo",
                "deny",
                "--config",
                "tools/config/deny.toml",
                "--locked",
                "--format",
                "json",
                "check",
                "advisories",
                "bans",
                "licenses",
                "sources",
            ],
        ),
        ("audit", ["cargo", "audit", "--json"]),
        ("pnpm", ["pnpm", "--dir", "crates/dokime-gui", "audit", "--prod", "--json"]),
        ("uv", ["uv", "audit", "--locked", "--output-format", "json"]),
    ]
    failures = []
    for name, command in commands:
        result = subprocess.run(
            command, check=False, cwd=directory, capture_output=True, text=True, encoding="utf-8", timeout=600, creationflags=spawn_flags()
        )
        (output / f"{name}.stdout.json").write_text(result.stdout, encoding="utf-8")
        (output / f"{name}.stderr.jsonl").write_text(result.stderr, encoding="utf-8")
        findings = []
        if name == "deny":
            for line in result.stderr.splitlines():
                event = json.loads(line)
                fields = event.get("fields", {})
                if fields.get("severity") != "error":
                    continue
                graphs = fields.get("graphs", [])
                package = graphs[0]["Krate"] if graphs else {"name": "unknown", "version": "unknown"}
                code = fields.get("code", "unknown")
                identifier = fields.get("advisory", {}).get("id", code)
                category = "advisory"
                if code == "rejected":
                    identifier = fields["labels"][0]["span"]
                    category = "license"
                if code == "wildcard":
                    category = "ban"
                entry = accepted(document, category, package["name"], package["version"], identifier)
                if entry and category == "ban":
                    spans = sorted(label["span"] for label in fields["labels"] if label["message"] == "wildcard dependency")
                    if spans != entry["spans"]:
                        entry = None
                findings.append((identifier, package["name"], package["version"], entry))
        elif name == "audit":
            report = json.loads(result.stdout)
            for item in report["vulnerabilities"]["list"]:
                identifier = item["advisory"]["id"]
                package = item["package"]
                entry = accepted(document, "advisory", package["name"], package["version"], identifier)
                findings.append((identifier, package["name"], package["version"], entry))
        elif name == "pnpm":
            report = json.loads(result.stdout)
            if "advisories" not in report or report.get("error"):
                raise policy.ToolError("pnpm audit returned no advisory report")
            for item in report["advisories"].values():
                identifier = item["github_advisory_id"]
                for version in sorted({finding["version"] for finding in item["findings"]}):
                    entry = accepted(document, "advisory", item["module_name"], version, identifier)
                    findings.append((identifier, item["module_name"], version, entry))
        if result.returncode and not findings:
            failures.append(f"{name}: tool failed with exit {result.returncode}; see {output}")
        for identifier, package, version, entry in findings:
            if entry is None:
                failures.append(f"{name}: unaccepted {identifier} in {package} {version}")
                continue
            print(
                f"{name}: accepted {identifier} in {package} {version}; {entry['reason']}; owner {entry['owner']}; expires {entry['expires']}"
            )
        print(f"{name}: native exit {result.returncode}; {len(findings)} findings")
    if failures:
        raise policy.ToolError("\n".join(failures))
    print("Supply-chain checks passed with documented exceptions")
    return 0


def auditable(profile: str, command: list[str]) -> int:
    """Apply the Pinned Cargo-Auditable Compiler Wrapper to Existing Build Tasks."""
    environment = dict(os.environ)
    if profile == "release":
        executable = shutil.which("cargo-auditable")
        if executable is None:
            raise policy.ToolError("the pinned cargo-auditable tool is missing")
        if environment.get("RUSTC_WORKSPACE_WRAPPER"):
            raise policy.ToolError("cargo-auditable requires ownership of RUSTC_WORKSPACE_WRAPPER")
        environment["RUSTC_WORKSPACE_WRAPPER"] = executable
        # cargo-auditable 0.7.7 Uses This Protocol for Its Rustc Wrapper.
        environment["CARGO_AUDITABLE_ORIG_ARGS"] = json.dumps({"offline": False, "locked": True, "frozen": False, "config": []})
    if command[:1] == ["--"]:
        command = command[1:]
    if command[:1] == ["python"]:
        command[0] = sys.executable
    if not command:
        raise policy.ToolError("pass an existing build entry point")
    result = subprocess.run(
        command,
        env=environment,
        check=False,
        creationflags=spawn_flags(),
        stdin=sys.stdin,
        stdout=sys.stdout,
        stderr=sys.stderr,
    )
    if result.returncode:
        print(f"Build Rust Products: {subprocess.list2cmdline(command)} Exited {result.returncode}", file=sys.stderr, flush=True)
    return result.returncode


def workspace(directory: Path, target: str, output: Path) -> list[Path]:
    """Run Cargo CycloneDX in an Isolated Source Snapshot, Preserving the Checkout."""
    output.mkdir(parents=True, exist_ok=True)
    tracked = policy.execute(directory, ["git", "ls-files", "-z"], capture=True).split("\0")
    with tempfile.TemporaryDirectory(prefix="dokime-sbom-") as temporary:
        snapshot = Path(temporary)
        for name in tracked:
            source = directory / name
            if not name or not source.is_file():
                continue
            if source.suffix not in {".rs", ".toml"} and source.name != "Cargo.lock":
                continue
            destination = snapshot / name
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(source, destination)
        policy.execute(snapshot, ["cargo", "cyclonedx", "--format", "json", "--spec-version", "1.5", "--all", "--target", target])
        if (snapshot / "Cargo.lock").read_bytes() != (directory / "Cargo.lock").read_bytes():
            raise policy.ToolError("Cargo CycloneDX changed the dependency resolution")
        result = []
        for source in sorted(snapshot.rglob("*.cdx.json")):
            relative = source.relative_to(snapshot)
            name = "-".join(relative.parts).removesuffix(".cdx.json") + f"-{target}.cdx.json"
            destination = output / name
            shutil.copyfile(source, destination)
            result.append(destination)
        if not result:
            raise policy.ToolError("Cargo CycloneDX produced no inventories")
        return result


def shell(directory: Path, output: Path) -> Path:
    """Scan the Locked Shell Dependency Inventory with Syft."""
    with tempfile.TemporaryDirectory(prefix="dokime-shell-sbom-") as temporary:
        snapshot = Path(temporary)
        for name in ("package.json", "pnpm-lock.yaml"):
            shutil.copyfile(directory / "crates/dokime-gui" / name, snapshot / name)
        document = json.loads(policy.execute(directory, ["syft", "scan", f"dir:{snapshot}", "-o", "cyclonedx-json"], capture=True))
    if not document.get("components"):
        raise policy.ToolError("Syft produced an empty shell inventory")
    output.mkdir(parents=True, exist_ok=True)
    destination = output / "dokime-shell.cdx.json"
    destination.write_text(json.dumps(document, indent=2) + "\n", encoding="utf-8")
    return destination


def unpack(archive: Path, destination: Path) -> None:
    """Reject Traversal and Links Before Extracting a Distribution."""
    if zipfile.is_zipfile(archive):
        with zipfile.ZipFile(archive) as bundle:
            for member in bundle.infolist():
                name = member.filename
                mode = member.external_attr >> 16
                if "\\" in name or ":" in name or not (destination / name).resolve().is_relative_to(destination.resolve()):
                    raise policy.ToolError(f"unsafe archive member: {name}")
                if mode & 0o170000 == 0o120000:
                    raise policy.ToolError(f"archive link is not supported: {name}")
            bundle.extractall(destination)
        return
    if tarfile.is_tarfile(archive):
        with tarfile.open(archive) as bundle:
            bundle.extractall(destination, filter="data")
        return
    shutil.copyfile(archive, destination / archive.name)


def artifact(directory: Path, source: Path, output: Path, rust: list[Path]) -> Path:
    """Scan Unpacked Bytes and Include the Target's Conservative Rust Source Inventory."""
    with tempfile.TemporaryDirectory(prefix="dokime-artifact-") as temporary:
        unpacked = Path(temporary)
        unpack(source, unpacked)
        document = json.loads(policy.execute(directory, ["syft", "scan", f"dir:{unpacked}", "-o", "cyclonedx-json"], capture=True))
    components = {item.get("purl", item.get("bom-ref", item["name"])): item for item in document.get("components", [])}
    for path in rust:
        for item in json.loads(path.read_text(encoding="utf-8")).get("components", []):
            key = item.get("purl", item.get("bom-ref", item["name"]))
            if key in components:
                continue
            item["bom-ref"] = key
            components[key] = item
    document["components"] = list(components.values())
    root_component = document.setdefault("metadata", {}).setdefault("component", {})
    root_component.update(
        {
            "type": "file",
            "name": source.name,
            "hashes": [{"alg": "SHA-256", "content": hashlib.sha256(source.read_bytes()).hexdigest()}],
            "properties": [
                {
                    "name": "dokime:inventory-scope",
                    "value": "Syft artifact scan plus conservative source inventories supplied for this target and artifact kind",
                }
            ],
        }
    )
    destination = output / (source.name + ".cdx.json")
    output.mkdir(parents=True, exist_ok=True)
    destination.write_text(json.dumps(document, indent=2) + "\n", encoding="utf-8")
    return destination


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--target")
    parser.add_argument("--check-supply", action="store_true")
    parser.add_argument("--auditable", choices=("dev", "release"))
    parser.add_argument("command", nargs=argparse.REMAINDER)
    parser.add_argument("--output", type=Path, default=Path("target/sbom"))
    parser.add_argument("--artifacts", type=Path)
    options = parser.parse_args()
    directory = policy.root()
    if options.check_supply:
        return check(directory)
    if options.auditable:
        return auditable(options.auditable, options.command)
    if not options.target:
        parser.error("--target is required for SBOM generation")
    inventories = workspace(directory, options.target, options.output)
    shell_inventory = shell(directory, options.output)
    if options.artifacts:
        sources = [
            source
            for source in sorted(options.artifacts.rglob("*"))
            if source.is_file() and source.name.endswith((".zip", ".tar.gz", ".whl", ".exe", ".deb", ".rpm"))
        ]
        if not sources:
            raise policy.ToolError("no distribution artifacts found")
        for source in sources:
            desktop = source.name.startswith("dokime-distribution-") or source.name.endswith(("-setup.exe", ".deb", ".rpm"))
            source_inventories = [*inventories, shell_inventory] if desktop else inventories
            artifact(directory, source, options.output, source_inventories)
    print(f"Generated {len(inventories)} Rust inventories for {options.target}")
    return 0


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    raise SystemExit(main())
