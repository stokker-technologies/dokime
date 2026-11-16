# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Own Release Staging Operations."""

# === Imports === #

# Standard Library Imports
import hashlib
import json
import shutil
import zipfile
from datetime import UTC, datetime
from pathlib import Path

# isort: split

# Third Party Imports

# isort: split

# Local Imports
import dokime_tools.package.definition
import dokime_tools.package.verification

from .. import policy, temporary
from ..build import paths, wheel
from ..docs import site
from ..package.changelog import notes
from . import context, contracts, layout, notices, reports, sbom

# Type Checking Imports

# === Public Exports === #

# === Constants === #


# === Types === #

# === Classes === #

# === Public Functions === #


def assets(directory: Path, settings: dict, selected: str, tag: str, profile: str | None = None, *, signed: bool = False) -> list[dict]:
    """The Declared Release Downloads of One Target, From Its Existing Package of the Profile."""
    profile = policy.profile() if profile is None else profile
    result = []
    tools = []
    for item in dokime_tools.package.definition.packaged(directory, profile, selected):
        tools.append({**item, "kind": "tool"})
    declared = settings.get("assets", []) + tools
    for definition in declared:
        profiles = definition.get("profiles", ["dev", "release"])
        if (
            not isinstance(profiles, list)
            or not profiles
            or any(profile not in ("dev", "release") for profile in profiles)
            or len(set(profiles)) != len(profiles)
        ):
            raise policy.ToolError("Stage Release: release.assets.profiles must be a nonempty array of distinct dev or release values")
        signed_only = definition.get("signed_only", False)
        if not isinstance(signed_only, bool):
            raise policy.ToolError("Stage Release: release.assets.signed_only must be a boolean")
        if definition.get("target") != selected:
            continue
        if profile not in profiles:
            continue
        if signed_only and not signed:
            continue
        relative = definition["path"].format(profile=profile, target=selected, product=settings["product"], version=tag[1:])
        source = policy.inside(directory, relative)
        if source.name == "latest.json" and not signed:
            continue
        name = layout.asset_name(directory, settings, definition, selected, tag, source)
        if name is None:
            continue
        reserved = {context.SUMS, context.STATE, context.INDEX, "CHANGELOG.md", "plugins.json", "upload.json", ".", ".."}
        if Path(name).name != name or name in reserved or "\\" in name or any(ord(character) < 32 for character in name):
            raise policy.ToolError(f"Stage Release: invalid release asset name: {name}")
        if not source.is_file() or source.is_symlink():
            raise policy.ToolError(f"Stage Release: missing regular release asset: {source}")
        result.append({"name": name, "path": relative, "target": selected})
    return result


def stage(
    directory: Path,
    settings: dict,
    tag: str,
    targets: list[str],
    profile: str | None = None,
    *,
    signed: bool = False,
    source_revision: str | None = None,
    release_notes: str | None = None,
    channel: str = "stable",
) -> list[dict]:
    """Copy Each Target's Release Downloads Into Its target/release-assets/<target>/ and Their SHA256SUMS Into any/."""
    profile = policy.profile() if profile is None else profile
    root = context.root(directory)
    distributions = []
    for selected in targets:
        distribution = (
            paths.output(directory, "package", profile, selected)
            / "rust"
            / f"{settings['product']}-distribution-{tag[1:]}-{layout.triple(directory, settings, selected)}.zip"
        )
        distributions.append(distribution)
    packaged_notices = notices.packaged(distributions)
    # Only the staged targets and the shared checksums are replaced; another target's staging stays untouched.
    for owned in [*targets, context.SHARED]:
        destination = policy.inside(root, owned)
        if destination == root or destination.is_symlink():
            raise policy.ToolError("Stage Release: unsafe release staging target")
        if destination.exists():
            shutil.rmtree(destination)
    shared = root / context.SHARED
    shared.mkdir(parents=True)
    for name, content in packaged_notices.items():
        (shared / name).write_bytes(content)
    python = paths.output(directory, "package", profile, context.SHARED) / "python"
    selected_python = wheel.distributions(directory, python, tag[1:])
    files = []
    for source in selected_python:
        destination = shared / source.name
        shutil.copyfile(source, destination)
        files.append(describe(root, destination))
    updater = {"version": tag[1:], "platforms": {}}
    plugins = layout.plugins(directory, settings, tag, targets, profile)
    for selected in targets:
        # Only the native target's executables can run here; other targets arrive verified by their Package jobs.
        if selected == context.native():
            dokime_tools.package.verification.verify(directory, profile, selected)
        (root / selected).mkdir(parents=True)
        declared = assets(directory, settings, selected, tag, profile, signed=signed)
        for asset in declared:
            if asset["name"] == "latest.json":
                metadata = json.loads(policy.inside(directory, asset["path"]).read_text(encoding="utf-8"))
                if metadata["version"] != tag[1:]:
                    raise policy.ToolError("Stage Release: updater metadata version differs from release")
                updater["platforms"].update(metadata["platforms"])
                continue
            # Downloads publish side by side, so a name must be unique across every target.
            if any(item["name"].casefold() == asset["name"].casefold() for item in files):
                raise policy.ToolError(f"Stage Release: release download names collide across targets: {asset['name']}")
            destination = policy.inside(root / selected, asset["name"])
            shutil.copyfile(policy.inside(directory, asset["path"]), destination)
            if destination.name.startswith(("dokime-cli-",)) and destination.name.endswith((".zip", ".tar.gz")):
                notices.archive(shared, destination)
            files.append(describe(root, destination))
    for destination in layout.plugin_archives(directory, root, plugins):
        notices.archive(shared, destination)
        if any(item["name"].casefold() == destination.name.casefold() for item in files):
            raise policy.ToolError(f"Stage Release: release download names collide across targets: {destination.name}")
        files.append(describe(root, destination))
    changelog = root / context.SHARED / "CHANGELOG.md"
    content = notes(directory, tag) if release_notes is None else release_notes
    if source_revision is not None:
        content += f"\nSource Commit: {source_revision}\n"
    changelog.write_text(content, encoding="utf-8", newline="\n")
    protocol = root / context.SHARED / f"dokime-proto-{tag[1:]}.zip"
    native = context.native()
    if native not in targets:
        raise policy.ToolError("Stage Release: protocol export requires a packaged native CLI target")
    cli = next(item for item in dokime_tools.package.definition.definitions(directory, profile, native) if item.get("command") == "dokime")
    temporary_root_directory = temporary.Directory()
    temporary_root = temporary_root_directory.path
    try:
        executable = temporary_root / cli["binary"]
        executable.write_bytes(dokime_tools.package.verification.binary_bytes(root / native / cli["output"], cli["binary"]))
        executable.chmod(0o755)
        # Linux's `dokime` executable must not occupy the exported `dokime/` package directory.
        exported = temporary_root / "proto"
        policy.execute(directory, [str(executable), "proto", "export", "--out", str(exported)])
        sources = sorted(exported.rglob("*.proto"))
        package_roots = []
        for package_roots_owner in ("dokime", "harmos"):
            for package_roots_audience in ("model", "host", "plugin"):
                package_roots.append(f"{package_roots_owner}/{package_roots_audience}/v1")

        missing = []
        for missing_package in package_roots:
            if (exported / missing_package).is_dir():
                continue
            missing.append(missing_package)

        if not sources:
            raise policy.ToolError("Stage Release: Export Release Protocol: No Protocol Definitions Were Produced")
        if missing:
            raise policy.ToolError(f"Stage Release: Export Release Protocol: Missing Definitions: {', '.join(missing)}")
        contracts.package(exported, protocol)
    finally:
        temporary_root_directory.close()
    files.append(describe(root, protocol))

    compliance = root / context.SHARED / f"dokime-compliance-{tag[1:]}.zip"
    shell_inventory = sbom.shell(directory, root / context.SHARED)
    archive = zipfile.ZipFile(compliance, "w", compression=zipfile.ZIP_DEFLATED)
    try:
        for name in ("NOTICE", "THIRD-PARTY-NOTICES"):
            archive.write(shared / name, name)
        archive.write(shell_inventory, "crates/any/" + shell_inventory.name)
        for selected in targets:
            rust = sbom.workspace(directory, layout.triple(directory, settings, selected), root / selected)
            for source in rust:
                archive.write(source, f"crates/{selected}/{source.name}")
            for item in files:
                source = root / item["path"]
                if item["path"].split("/")[0] not in (selected, context.SHARED):
                    continue
                if selected != targets[0] and item["path"].split("/")[0] == context.SHARED:
                    continue
                desktop = source.name.startswith("dokime-distribution-") or source.name.endswith(("-setup.exe", ".deb", ".rpm"))
                inventories = [*rust, shell_inventory] if desktop else rust
                if source.parent == shared and source.name.endswith((".whl", ".tar.gz")):
                    inventories = []
                destination = sbom.artifact(directory, source, source.parent, inventories)
                archive.write(destination, "artifacts/" + destination.name)
    finally:
        archive.close()
    files.append(describe(root, compliance))
    documentation = _documentation(directory, root, tag, channel, required=channel == "stable" and signed)
    if documentation is not None:
        files.append(describe(root, documentation))
    if channel == "nightly":
        for destination in reports.stage(directory, shared, tag[1:]):
            files.append(describe(root, destination))
    executables = {}
    for executables_item in policy.config(directory).get("package", {}).get("executables", []):
        executables[executables_item["name"]] = "cli" if executables_item["command"] == settings["product"] else executables_item["command"]

    manifest = layout.downloads(
        settings,
        executables,
        version=tag[1:],
        channel=channel,
        tag=tag if channel == "stable" else channel,
        published=datetime.now(UTC).strftime("%Y-%m-%dT%H:%M:%SZ"),
        files=files,
        sums="".join(f"{item['sha256']}  {item['name']}\n" for item in files),
    )
    # Tauri Reads The Root Updater Fields; The Other Consumers Read Named Sections.
    document = {
        **updater,
        "notes": content,
        "pub_date": manifest["pub_date"],
        "downloads": manifest,
        "plugins": layout.plugin_feed(settings, tag, plugins),
    }
    policy.write_json(root / context.SHARED / context.INDEX, document)
    files.append(describe(root, root / context.SHARED / context.INDEX))
    (root / context.SHARED / context.SUMS).write_text("".join(f"{item['sha256']}  {item['name']}\n" for item in files), encoding="utf-8")
    files.append(describe(root, root / context.SHARED / context.SUMS))
    layout.upload_manifest(directory, root, files)
    return files


def checksums(path: Path, inventory: dict) -> None:
    expected = {}
    for expected_name, expected_item in inventory.items():
        if expected_name == context.SUMS:
            continue
        expected[expected_name] = expected_item["sha256"]

    actual = {}
    for line in path.read_text(encoding="utf-8").splitlines():
        digest, separator, name = line.partition("  ")
        if not separator or name in actual:
            raise policy.ToolError("Stage Release: invalid or duplicate SHA256SUMS entry")
        actual[name] = digest
    if actual != expected:
        raise policy.ToolError("Stage Release: SHA256SUMS differs from upload.json")


def verified_bytes(root: Path, item: dict) -> bytes:
    data = policy.inside(root, item["path"]).read_bytes()
    if hashlib.sha256(data).hexdigest() != item["sha256"]:
        raise policy.ToolError(f"Stage Release: staged release bytes changed: {item['name']}")
    return data


def describe(root: Path, path: Path) -> dict:
    data = path.read_bytes()
    return {"name": path.name, "path": path.relative_to(root).as_posix(), "sha256": hashlib.sha256(data).hexdigest(), "size": len(data)}


# === Private Functions === #


def _documentation(directory: Path, root: Path, tag: str, channel: str, *, required: bool) -> Path | None:
    """Stage the Built Documentation Archive of Exactly This Source; Stable Publication Requires It."""
    documentation = f"{site.OUTPUT}/{site.ARCHIVE}"
    source = policy.inside(directory, documentation)
    if not source.is_file() or source.is_symlink():
        if required:
            raise policy.ToolError(
                f"Publish Release: The Documentation Archive Is Required; Build It With mise run docs:build -- --release --archive ({documentation})"
            )
        return None
    archive = zipfile.ZipFile(source)
    try:
        stamp = json.loads(archive.read(site.STAMP))
    finally:
        archive.close()
    major, minor, _ = tag[1:].split(".")
    entry = "dev" if channel == "nightly" else f"{major}.{minor}"
    if stamp.get("entry") != entry or (channel != "nightly" and stamp.get("version") != tag[1:]):
        raise policy.ToolError(
            f"Stage Documentation: Archive Holds Entry {stamp.get('entry')} {stamp.get('version')}, Expected {entry} {tag[1:]}; "
            f"Rebuild It With mise run docs:build -- {'' if channel == 'nightly' else '--release '}--archive"
        )
    destination = root / context.SHARED / f"dokime-docs-{'nightly' if channel == 'nightly' else tag[1:]}.zip"
    shutil.copyfile(source, destination)
    return destination


# === Entry Point === #
