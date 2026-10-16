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
import re
import shutil
import subprocess
from pathlib import Path, PurePosixPath
from typing import Any
from xml.etree import ElementTree

# Third Party Imports
# Local Imports
from dokime_tools import contracts, release
from dokime_tools.check.wheel import verify_wheel
from dokime_tools.fixtures import FIXTURES, STAMP

# Type Checking Imports

# === Public Exports === #

# === Constants === #

PLATFORMS = ("linux", "windows")
TARGETS = {"linux": "x86_64-unknown-linux-gnu", "windows": "x86_64-pc-windows-msvc"}
BINARIES = "bin"
PROTO = "proto"
VALIDATION = "validation"
REPORT_TEMPLATE = f"{VALIDATION}/report-template.html"
AUDITS = f"{VALIDATION}/audits"
FIXTURE_BUNDLES = f"{VALIDATION}/fixtures"
TEST_EVIDENCE = f"{VALIDATION}/tests"
COVERAGE_EVIDENCE = f"{VALIDATION}/coverage"
EXAMPLES = "examples"
EXAMPLE_REPORT = "charger-report.html"
# Every role lives in exactly one package directory; the prefix of a `kind:name`
# role selects its family.
DIRECTORIES = {
    "license": "licenses",
    "host": BINARIES,
    "client": BINARIES,
    "tool": BINARIES,
    "desktop": BINARIES,
    "plugin": "plugins",
    "wheel": "python",
    "sdist": "python",
    "deb": "installers",
    "rpm": "installers",
    "installer": "installers",
    "signature": "installers",
    "updater": "installers",
    "wheel-audit": AUDITS,
    "timing": VALIDATION,
    "proto": PROTO,
    "fixture": FIXTURE_BUNDLES,
    "test": TEST_EVIDENCE,
    "coverage": COVERAGE_EVIDENCE,
    "example": EXAMPLES,
}
# Only Python distributions may nest an archive inside the download; the
# Nextest archives travel as their own CI artifact bound by `test_archives`.
ARCHIVE_SUFFIXES = (".tar", ".tar.gz", ".tgz", ".tar.zst", ".zip")
ARCHIVE_DIRECTORIES = ("python",)
TEST_ARCHIVES = "target/test-archives"

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


def build_profile() -> str:
    profile = os.environ.get("STKR_TEST_PROFILE", "dev")
    if profile not in {"dev", "release"}:
        raise ValueError("STKR_TEST_PROFILE must be dev or release")
    return profile


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
            expected_profile="release",
        )
        for platform in PLATFORMS
    ]
    if any(
        manifest["package_version"] != tag[1:] or manifest["build_revision"] != commit
        for manifest in manifests
    ):
        raise ValueError("tag version differs from the already validated packages")
    return manifests


def required_suites(platform: str, profile: str = "release") -> tuple[str, ...]:
    if platform not in PLATFORMS:
        raise ValueError("unknown native validation platform")
    suites = (
        "integration-rust",
        "integration-python",
        "integration-typescript",
        "acceptance-rust",
        "acceptance-python",
    )
    if profile not in {"dev", "release"}:
        raise ValueError("unknown native validation profile")
    return suites + (
        ("system-typescript",) if platform == "windows" and profile == "release" else ()
    )


def verify_platform(
    root: Path,
    platform: str,
    source: Path,
    *,
    require_validation: bool = False,
    check_tools: bool = True,
    expected_profile: str | None = None,
) -> dict:
    manifest = json.loads(_file(root, "manifest.json").read_text(encoding="utf-8"))
    identity = inputs(source)
    if (
        not identity.get("source_tree")
        or manifest.get("version") != 1
        or manifest.get("platform") != platform
        or manifest.get("inputs") != identity
        or manifest.get("build_revision") != identity["revision"]
        or manifest.get("profile") not in {"dev", "release"}
        or (
            expected_profile is not None and manifest.get("profile") != expected_profile
        )
        or manifest.get("target") != TARGETS[platform]
        or manifest.get("architecture") != "x86_64"
        or manifest.get("python") != "py3"
        or manifest.get("abi") != "none"
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
        "client",
        "tool",
        "desktop",
        "report-template",
        "plugin",
        "wheel",
        "wheel-audit",
    }
    if manifest["profile"] == "release":
        required.add("sdist")
        required |= {"deb", "rpm"} if platform == "linux" else {"installer"}
    elif {"sdist", "deb", "rpm", "installer", "signature", "updater"} & by_role.keys():
        raise ValueError("development artifacts must not include release distributions")
    required |= {f"fixture:{name}" for name, _, _ in FIXTURES} | {f"fixture:{STAMP}"}
    required |= {f"proto:{name}" for name in contracts.CORE_MEMBERS}
    if not required <= by_role.keys():
        raise ValueError(f"incomplete {platform} release artifact roles")
    _layout(by_role)
    _test_archives(manifest)
    if by_role["license"]["path"] != "licenses/LICENSE-APACHE":
        raise ValueError("canonical Apache license is missing")
    if by_role["report-template"]["path"] != REPORT_TEMPLATE:
        raise ValueError("canonical report template is missing")
    _file(root, by_role["report-template"]["path"]).read_text(encoding="utf-8")
    _proto(root, manifest, by_role)
    version = manifest["package_version"]
    installer_names = (
        {"deb": f"dokime_{version}_amd64.deb", "rpm": f"dokime-{version}-1.x86_64.rpm"}
        if platform == "linux"
        else {"installer": f"dokime-{version}-windows-x86_64-setup.exe"}
    )
    for role, expected in (
        installer_names.items() if manifest["profile"] == "release" else ()
    ):
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
    if filename != f"dokime-{version}-py3-none-any.whl":
        raise ValueError("release wheel name disagrees with version or purity")
    if (
        "sdist" in by_role
        and PurePosixPath(by_role["sdist"]["path"]).name != f"dokime-{version}.tar.gz"
    ):
        raise ValueError("release source archive must be the dokime distribution")
    actual_audit = verify_wheel(_file(root, wheel["path"]))["wheel-audit"]
    stored_audit = _file(root, by_role["wheel-audit"]["path"]).read_text(
        encoding="utf-8"
    )
    if json.loads(stored_audit) != json.loads(actual_audit):
        raise ValueError("wheel audit differs from the actual packaged artifact")
    validation = manifest.get("validation")
    if require_validation:
        suites = set(required_suites(platform, manifest["profile"]))
        if (
            not isinstance(validation, dict)
            or set(validation.get("suites", [])) != suites
        ):
            raise ValueError("release requires complete native test evidence")
        _coverage(root, manifest, by_role)
        _example(root, by_role)
        for suite in suites:
            name = f"test:tests-{suite}.xml"
            if name not in by_role:
                raise ValueError(f"missing native test report: {suite}")
            _passed_tests(_file(root, by_role[name]["path"]))


def _layout(roles: dict[str, dict]) -> None:
    for role, artifact in roles.items():
        family = role.split(":", 1)[0] if ":" in role else role
        path = PurePosixPath(artifact["path"])
        if role == "report-template":
            directory = VALIDATION
        elif family in DIRECTORIES:
            directory = DIRECTORIES[family]
        else:
            raise ValueError(
                f"release artifact role is outside the package layout: {role}"
            )
        nested = family in {"fixture", "proto"}
        if (
            str(path.parent) != directory
            if not nested
            else directory not in {str(parent) for parent in path.parents}
        ):
            raise ValueError(
                f"release artifact is outside its package directory: {role}"
            )
        if (
            path.name.endswith(ARCHIVE_SUFFIXES)
            and directory not in ARCHIVE_DIRECTORIES
        ):
            raise ValueError(f"release download must not nest an archive: {role}")


def _test_archives(manifest: dict) -> None:
    """The Nextest archives the build job uploaded beside this package."""
    archives = manifest.get("test_archives")
    local = manifest.get("profile") == "dev" and all(
        manifest.get(key) is None
        for key in ("build_job_id", "pipeline_id", "run_attempt")
    )
    artifact = archives.get("artifact") if isinstance(archives, dict) else None
    valid_identity = (
        artifact is None
        if local
        else isinstance(artifact, str)
        and re.fullmatch(rf"test-archives-{manifest['platform']}-[1-9][0-9]*", artifact)
    )
    if (
        not isinstance(archives, dict)
        or not valid_identity
        or not isinstance(archives.get("files"), list)
        or not archives["files"]
    ):
        raise ValueError("release test archive artifact identity is missing")
    names = []
    for entry in archives["files"]:
        if (
            not isinstance(entry, dict)
            or not isinstance(entry.get("name"), str)
            or not re.fullmatch(r"[A-Za-z0-9_.-]+", entry["name"])
            or entry["name"] in {".", ".."}
            or not isinstance(entry.get("sha256"), str)
            or not re.fullmatch(r"[a-f0-9]{64}", entry["sha256"])
            or type(entry.get("bytes")) is not int
            or entry["bytes"] < 0
        ):
            raise ValueError("release test archive inventory is invalid")
        names.append(entry["name"])
    bundles = {
        name.removesuffix(".tar.zst") for name in names if name.endswith(".tar.zst")
    }
    if (
        len(names) != len(set(names))
        or not bundles
        or set(names)
        != {name + suffix for name in bundles for suffix in (".tar.zst", ".tar.json")}
    ):
        raise ValueError(
            "release preparation requires complete Nextest archives and receipts"
        )


def verify_test_archives(root: Path, manifest: dict) -> Path:
    """Downloaded Nextest archives must be exactly the candidate's own bytes."""
    directory = root / TEST_ARCHIVES
    if directory.is_symlink() or not directory.is_dir():
        raise ValueError(f"missing test archives: {directory}")
    expected = {entry["name"]: entry for entry in manifest["test_archives"]["files"]}
    actual = {path.name for path in directory.iterdir()}
    if actual != set(expected):
        raise ValueError("downloaded test archives differ from the candidate's own")
    for name, entry in expected.items():
        path = directory / name
        if (
            path.is_symlink()
            or not path.is_file()
            or path.stat().st_size != entry["bytes"]
            or digest(path) != entry["sha256"]
        ):
            raise ValueError(f"test archive checksum or size mismatch: {name}")
    return directory


def _proto(root: Path, manifest: dict, roles: dict) -> None:
    exported = {
        role.removeprefix("proto:"): artifact
        for role, artifact in roles.items()
        if role.startswith("proto:")
    }
    names = (
        _file(root, exported["manifest.txt"]["path"])
        .read_text(encoding="utf-8")
        .splitlines()
    )
    if (
        len(names) != len(set(names))
        or set(exported) != set(names) | {"manifest.txt", "revision.txt"}
        or any(
            name not in contracts.CORE_MEMBERS and not name.endswith(".proto")
            for name in names
        )
        or any(
            artifact["path"] != f"{PROTO}/{name}" for name, artifact in exported.items()
        )
    ):
        raise ValueError("release protocol files differ from their contract manifest")
    revision = _file(root, exported["revision.txt"]["path"]).read_text(encoding="ascii")
    if revision != manifest["build_revision"] + "\n":
        raise ValueError("release protocol files belong to another source commit")
    archive = manifest.get("contracts", {}).get("archive_sha256")
    if not isinstance(archive, str) or not re.fullmatch(r"[a-f0-9]{64}", archive):
        raise ValueError("release protocol archive identity is missing")


def _example(root: Path, roles: dict) -> None:
    entry = roles.get(f"example:{EXAMPLE_REPORT}")
    if entry is None or entry["path"] != f"{EXAMPLES}/{EXAMPLE_REPORT}":
        raise ValueError("release requires the acceptance run's example report")
    if not _file(root, entry["path"]).read_text(encoding="utf-8").strip():
        raise ValueError("release example report is empty")


def _coverage(root: Path, manifest: dict, roles: dict) -> None:
    from dokime_tools import evidence

    entry = roles.get("coverage:coverage.json")
    if entry is None:
        raise ValueError("release requires the distinct required coverage lane")
    proof = json.loads(_file(root, entry["path"]).read_text(encoding="utf-8"))
    validation = manifest["validation"]
    suites = {
        "integration-rust",
        "acceptance-rust",
        "integration-python",
        "acceptance-python",
    }
    if (
        proof.get("version") != 1
        or proof.get("platform") != manifest["platform"]
        or proof.get("inputs") != manifest["inputs"]
        or proof.get("pipeline_id") != manifest["pipeline_id"]
        or proof.get("run_attempt") != manifest["run_attempt"]
        or proof.get("build_manifest_sha256") != validation.get("build_manifest_sha256")
        or proof.get("coverage_job_id") != validation.get("coverage_job_id")
        or set(proof.get("suites", [])) != suites
    ):
        raise ValueError("release coverage proof belongs to different native inputs")
    reports = proof.get("files")
    if not isinstance(reports, list):
        raise TypeError("release coverage report inventory is missing")
    names = set()
    for report in reports:
        name = report.get("path")
        role = roles.get(f"coverage:{name}")
        if (
            not isinstance(name, str)
            or name in names
            or role is None
            or role["sha256"] != report.get("sha256")
            or role["bytes"] != report.get("bytes")
        ):
            raise ValueError("release coverage report inventory differs")
        names.add(name)
    if (
        not {"latest.json", "coverage-rust.xml", "coverage-python.xml"}
        | {f"tests-{suite}.xml" for suite in suites}
        <= names
    ):
        raise ValueError("release coverage omits required native reports")
    result = json.loads(
        _file(root, roles["coverage:latest.json"]["path"]).read_text(encoding="utf-8")
    )
    tests = result.get("tests")
    if (
        not isinstance(tests, dict)
        or set(tests) != suites
        or any(
            entry.get("outcome") != "passed"
            or entry.get("coverage") is not True
            or entry.get("revision") != manifest["build_revision"]
            or entry.get("platform") != manifest["platform"]
            or entry.get("requested_configuration") is not None
            for entry in tests.values()
        )
    ):
        raise ValueError("release requires complete distinct native coverage evidence")
    for suite in suites:
        _passed_tests(_file(root, roles[f"coverage:tests-{suite}.xml"]["path"]))
    for language in ("rust", "python"):
        evidence.validate_coverage(
            _file(root, roles[f"coverage:coverage-{language}.xml"]["path"]).read_text(
                encoding="utf-8"
            )
        )


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


# === Entry Point === #

if __name__ == "__main__":
    raise SystemExit(main())
