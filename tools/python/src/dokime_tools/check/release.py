# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Reconcile Both Native Release Artifact Manifests."""

# === Imports === #

# Standard Library Imports
import argparse
import hashlib
import json
import os
import subprocess
import re
import shutil
from pathlib import Path, PurePosixPath
from typing import Any
from xml.etree import ElementTree

# Third Party Imports

# Local Imports
from dokime_tools import release

# Type Checking Imports

# === Public Exports === #

# === Constants === #

PLATFORMS = ("linux", "windows")
TARGETS = {"linux": "x86_64-unknown-linux-gnu", "windows": "x86_64-pc-windows-msvc"}

# === Types === #

# === Classes === #

# === Public Functions === #


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path)
    parser.add_argument("--commit", required=True)
    parser.add_argument("--tag", required=True)
    args = parser.parse_args()
    verify_release(args.directory, args.commit, args.tag, Path.cwd())
    return 0


def verify_release(directory: Path, commit: str, tag: str, source: Path) -> list[dict]:
    if not re.fullmatch(r"[a-f0-9]{40}", commit) or not re.fullmatch(
        r"v\d+\.\d+\.\d+", tag
    ):
        raise ValueError("release requires a full commit and stable version tag")
    manifests = [
        verify_platform(
            directory / platform,
            platform,
            source,
            require_validation=True,
            check_tools=False,
        )
        for platform in PLATFORMS
    ]
    if any(
        manifest["package_version"] != tag[1:] or manifest["build_revision"] != commit
        for manifest in manifests
    ):
        raise ValueError("tag version differs from the already validated packages")
    return manifests


def verify_platform(
    root: Path,
    platform: str,
    source: Path,
    *,
    require_validation: bool = False,
    check_tools: bool = True,
) -> dict:
    manifest = json.loads(_file(root, "manifest.json").read_text(encoding="utf-8"))
    identity = inputs(source)
    if (
        not identity.get("source_tree")
        or manifest.get("version") != 1
        or manifest.get("platform") != platform
        or manifest.get("inputs") != identity
        or manifest.get("build_revision") != identity["revision"]
        or manifest.get("profile") != "release"
        or manifest.get("target") != TARGETS[platform]
        or manifest.get("architecture") != "x86_64"
        or manifest.get("python") != "cp313"
        or manifest.get("abi") != "abi3"
    ):
        raise ValueError(
            "release source, resolved inputs or platform identity mismatch"
        )
    tools = manifest.get("tools", {})
    names = {"mise", "rustc", "cargo", "python", "uv", "node", "pnpm"}
    if (
        not isinstance(tools, dict)
        or any(
            not isinstance(tools.get(name), str) or not tools[name] for name in names
        )
        or f"host: {TARGETS[platform]}"
        not in manifest.get("toolchain", "").splitlines()
    ):
        raise ValueError("release native toolchain identity is missing")
    if check_tools:
        for name in ("rustc", "cargo", "python", "uv", "node", "pnpm"):
            executable = shutil.which(name)
            if executable is None:
                raise FileNotFoundError(
                    f"{name} is not installed; install it through mise"
                )
            actual = subprocess.run(
                [executable, "--version"],
                cwd=source,
                capture_output=True,
                text=True,
                check=True,
                encoding="utf-8",
            ).stdout.strip()
            if tools.get(name) != actual:
                raise ValueError(f"release toolchain differs: {name}")
    _artifacts(root, manifest, source, require_validation=require_validation)
    return manifest


def inputs(root: Path) -> dict:
    identity = release.inputs(root)
    identity["revision"] = source_revision(root)
    return identity


def source_revision(root: Path) -> str:
    return subprocess.check_output(
        ["git", "rev-parse", "HEAD"], cwd=root, text=True, encoding="utf-8"
    ).strip()


def digest(path: Path) -> str:
    with path.open("rb") as content:
        return hashlib.file_digest(content, "sha256").hexdigest()


def release_tag(root: Path) -> str:
    from dokime_tools.ci.route import version

    return "v" + version(root)


def verify_source_identity(commit: str, tag: str) -> None:
    from dokime_tools.ci.route import verify

    if (
        os.environ.get("GITHUB_REF_PROTECTED") != "true"
        or os.environ.get("GITHUB_REF") != "refs/heads/main"
        or os.environ.get("GITHUB_REPOSITORY") != "stokker-technologies/dokime"
        or os.environ.get("GITHUB_EVENT_NAME") != "pull_request"
        or not re.fullmatch(r"v\d+\.\d+\.\d+", tag)
    ):
        raise ValueError("publication requires an approved merge into protected main")
    root = Path.cwd()
    event = json.loads(
        Path(os.environ["GITHUB_EVENT_PATH"]).read_text(encoding="utf-8")
    )
    request = event["pull_request"]
    if (
        event.get("action") != "closed"
        or request.get("merged") is not True
        or request["base"]["ref"] != "main"
        or request.get("merge_commit_sha") != commit
        or release_tag(root) != tag
        or _git_output(["git", "rev-parse", "HEAD"]) != commit
    ):
        raise ValueError("release checkout does not match the approved final commit")
    _git_output(["git", "merge-base", "--is-ancestor", request["head"]["sha"], commit])
    verify(event, root)
    if _git_output(["git", "status", "--porcelain", "--untracked-files=no"]):
        raise ValueError("release checkout has modified tracked sources")


# === Private Functions === #


def _git_output(command: list[str]) -> str:
    return subprocess.run(
        command, check=True, capture_output=True, text=True, encoding="utf-8"
    ).stdout.strip()


def _file(root: Path, name: str) -> Path:
    relative = PurePosixPath(name)
    if (
        not name
        or "\\" in name
        or ":" in name
        or relative.is_absolute()
        or ".." in relative.parts
        or str(relative) != name
    ):
        raise ValueError("release artifact paths must be canonical relative paths")
    path = root / name
    if root.is_symlink() or any(
        (root / Path(*relative.parts[:i])).is_symlink()
        for i in range(1, len(relative.parts) + 1)
    ):
        raise ValueError("release artifacts must not use symbolic links")
    if not path.is_file():
        raise ValueError(f"missing release artifact: {path}")
    return path


def _artifacts(
    root: Path, manifest: dict[str, Any], source: Path, *, require_validation: bool
) -> None:
    artifacts = manifest.get("files")
    if not isinstance(artifacts, list) or not artifacts:
        raise ValueError("release artifact inventory is empty")
    by_role = {}
    paths = {"manifest.json"}
    for artifact in artifacts:
        name = artifact["path"]
        role = artifact["role"]
        if name in paths or role in by_role:
            raise ValueError("duplicate release artifact path or role")
        path = _file(root, name)
        if (
            type(artifact["bytes"]) is not int
            or artifact["bytes"] < 0
            or path.stat().st_size != artifact["bytes"]
            or digest(path) != artifact["sha256"]
        ):
            raise ValueError(f"release artifact checksum or size mismatch: {name}")
        paths.add(name)
        by_role[role] = artifact
    actual = {
        path.relative_to(root).as_posix()
        for path in root.rglob("*")
        if path.is_file() or path.is_symlink()
    }
    if actual != paths:
        raise ValueError("release directory contains unlisted artifacts")
    platform = manifest["platform"]
    required = {
        "license",
        "host",
        "tool",
        "desktop",
        "report-template",
        "plugin",
        "protocol",
        "protocol-checksum",
        "wheel",
        "sdist",
        "abi3-audit",
    }
    required |= (
        {"deb", "rpm", "linux-audit", "wheel-build"}
        if platform == "linux"
        else {"installer"}
    )
    if not required <= by_role.keys():
        raise ValueError(f"incomplete {platform} release artifact roles")
    if by_role["license"]["path"] != "licenses/LICENSE-APACHE":
        raise ValueError("canonical Apache license is missing")
    if by_role["report-template"]["path"] != "reports/report.html":
        raise ValueError("canonical report template is missing")
    _file(root, by_role["report-template"]["path"]).read_text(encoding="utf-8")
    version = manifest["package_version"]
    installer_names = (
        {"deb": f"dokime_{version}_amd64.deb", "rpm": f"dokime-{version}-1.x86_64.rpm"}
        if platform == "linux"
        else {"installer": f"dokime-{version}-windows-x86_64-setup.exe"}
    )
    for role, expected in installer_names.items():
        if PurePosixPath(by_role[role]["path"]).name != expected:
            raise ValueError(
                "installer filename disagrees with package version or platform"
            )
    if {"signature", "updater"} & by_role.keys():
        if platform != "windows" or not {"signature", "updater"} <= by_role.keys():
            raise ValueError("incomplete signed updater artifacts")
        _updater(root, by_role, "v" + manifest["package_version"])
    wheel = by_role["wheel"]
    filename = PurePosixPath(wheel["path"]).name
    version = manifest["package_version"].replace("-dev.0", ".dev0")
    suffix = (
        "win_amd64"
        if platform == "windows"
        else r"manylinux_2_(?:[5-9]|1\d|2[0-8])_x86_64"
    )
    if not re.fullmatch(
        rf"dokime-{re.escape(version)}-cp313-abi3-{suffix}\.whl", filename
    ):
        raise ValueError("release wheel name disagrees with version, ABI, or platform")
    if PurePosixPath(by_role["sdist"]["path"]).name != f"dokime-{version}.tar.gz":
        raise ValueError(
            "release source archive must be the unified dokime distribution"
        )
    _abi3_report(_file(root, by_role["abi3-audit"]["path"]), filename)
    if platform == "linux":
        from dokime_tools.package.linux import verify

        build = json.loads(
            _file(root, by_role["wheel-build"]["path"]).read_text(encoding="utf-8")
        )
        # Match The Actual Container Toolchain And Both Packaged Inputs.
        verify(
            build,
            wheel["sha256"],
            by_role["sdist"]["sha256"],
            manifest["tools"]["rustc"],
            root=source,
        )
        report = json.loads(
            _file(root, by_role["linux-audit"]["path"]).read_text(encoding="utf-8")
        )
        floor = re.fullmatch(
            r"manylinux_2_(\d+)_x86_64", str(report.get("overall_tag"))
        )
        claimed = int(filename.split("manylinux_2_", 1)[1].split("_", 1)[0])
        if (
            report.get("version") != 1
            or report.get("wheel") != filename
            or report.get("pure") is not False
            or report.get("unsupported_isa") is not False
            or floor is None
            or int(floor[1]) > claimed
        ):
            raise ValueError("Linux audit does not prove the wheel portability claim")
    validation = manifest.get("validation")
    if require_validation:
        suites = {
            "integration-rust",
            "integration-python",
            "integration-javascript",
            "acceptance-rust",
            "acceptance-python",
        }
        if (
            not isinstance(validation, dict)
            or set(validation.get("suites", [])) != suites
        ):
            raise ValueError("release requires complete native test evidence")
        for suite in suites:
            name = f"test:tests-{suite}.xml"
            if name not in by_role:
                raise ValueError(f"missing native test report: {suite}")
            _passed_tests(_file(root, by_role[name]["path"]))


def _updater(root: Path, artifacts: dict, tag: str) -> None:
    project = "https://github.com/stokker-technologies/dokime"
    installer = PurePosixPath(artifacts["installer"]["path"]).name
    signature = (
        _file(root, artifacts["signature"]["path"]).read_text(encoding="utf-8").strip()
    )
    metadata = json.loads(
        _file(root, artifacts["updater"]["path"]).read_text(encoding="utf-8")
    )
    expected = {
        "windows-x86_64": {
            "signature": signature,
            "url": f"{project}/releases/download/{tag}/{installer}",
        }
    }
    if (
        not signature
        or not isinstance(metadata, dict)
        or metadata.get("version") != tag[1:]
        or metadata.get("platforms") != expected
        or PurePosixPath(artifacts["updater"]["path"]).name != "latest.json"
        or PurePosixPath(artifacts["signature"]["path"]).name != f"{installer}.sig"
    ):
        raise ValueError(
            "updater must identify the exact tagged installer and signature"
        )


def _passed_tests(path: Path) -> None:
    root = ElementTree.parse(path).getroot()
    cases = list(root.iter("testcase"))
    if not cases or any(
        case.find("failure") is not None or case.find("error") is not None
        for case in cases
    ):
        raise ValueError("release validation has missing or failed tests")
    if all(case.find("skipped") is not None for case in cases):
        raise ValueError("release validation cannot consist only of skipped tests")
    for suite in root.iter("testsuite"):
        if int(suite.get("failures", "0")) or int(suite.get("errors", "0")):
            raise ValueError("release validation reports failures or errors")


def _abi3_report(path: Path, filename: str) -> None:
    specs = json.loads(path.read_text(encoding="utf-8"))["specs"]
    if len(specs) != 1:
        raise ValueError("ABI3 audit must identify exactly one wheel")
    name, spec = next(iter(specs.items()))
    if (
        PurePosixPath(name.replace("\\", "/")).name != filename
        or spec.get("kind") != "wheel"
    ):
        raise ValueError("ABI3 audit identifies a different artifact")
    modules = spec.get("wheel", [])
    expected_module = "_native.pyd" if "win_amd64" in filename else "_native.abi3.so"
    if len(modules) != 1 or modules[0].get("name") != expected_module:
        raise ValueError("ABI3 audit must cover the native extension")
    result = modules[0].get("result", {})
    if (
        result.get("is_abi3") is not True
        or result.get("is_abi3_baseline_compatible") is not True
        or result.get("baseline") != "3.13"
        or result.get("non_abi3_symbols")
        or result.get("future_abi3_objects")
    ):
        raise ValueError("ABI3 audit failed the CPython 3.13 contract")


# === Entry Point === #

if __name__ == "__main__":
    raise SystemExit(main())
