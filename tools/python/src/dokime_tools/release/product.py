# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Check And Synchronize Version Metadata"""

# === Imports === #

# Standard Library Imports

import argparse
import json
import os
import re
import subprocess
import sys
import tomllib
from pathlib import Path

# Third Party Imports

# Local Imports

# Type Checking Imports


# === Public Exports === #

# === Constants === #

ROOT = Path(__file__).resolve().parents[5]
RELEASE = re.compile(r"^\d+\.\d+\.\d+$")
VERSION = RELEASE


# === Types === #

# === Classes === #


# === Public Functions === #


def python_version(version: str) -> str:
    if not RELEASE.fullmatch(version):
        raise ValueError("versions must use stable X.Y.Z form")
    return version


def check(root: Path = ROOT) -> None:
    manifest = _toml(root / "Cargo.toml")
    version = manifest["workspace"]["package"]["version"]
    if not VERSION.fullmatch(version):
        raise ValueError(f"unsupported Cargo workspace version: {version!r}")
    names = _package_names(root, manifest)
    errors: list[str] = []

    def expect(actual: object, expected: object, label: str) -> None:
        if actual != expected:
            errors.append(f"{label} is {actual!r}; expected {expected!r}")

    expect(
        _toml(root / "pyproject.toml")["project"]["version"],
        python_version(version),
        "pyproject.toml project.version",
    )
    sdk_path = root / "packages/dokime/pyproject.toml"
    sdk = _toml(sdk_path)["project"]
    expect(sdk.get("name"), "dokime", "Python distribution name")
    expect(sdk.get("version"), python_version(version), "Python client version")
    for relative in [
        "crates/dokime-gui/package.json",
        "crates/dokime-gui/src-tauri/tauri.conf.json",
    ]:
        expect(
            json.loads((root / relative).read_text(encoding="utf-8"))["version"],
            version,
            f"{relative} version",
        )
    tauri = _toml(root / "crates/dokime-gui/src-tauri/Cargo.toml")["package"]
    expect(tauri.get("name"), "dokime-gui", "Tauri package.name")
    expect(tauri.get("version"), {"workspace": True}, "Tauri package.version")
    lock = _toml(root / "Cargo.lock")["package"]
    for name in sorted(names):
        versions = [
            p.get("version") for p in lock if p["name"] == name and "source" not in p
        ]
        expect(versions, [version], f"Cargo.lock package {name}")
    uv = _toml(root / "uv.lock")["package"]
    workspace = next((p for p in uv if p["name"] == "dokime-rust-workspace"), {})
    expect(
        workspace.get("version"), python_version(version), "uv.lock workspace version"
    )
    sdk_lock = next((p for p in uv if p["name"] == "dokime"), {})
    expect(
        sdk_lock.get("source", {}).get("editable"),
        "packages/dokime",
        "uv.lock dokime source",
    )
    expect(sdk_lock.get("version"), python_version(version), "uv.lock dokime version")
    changelog = (root / "CHANGELOG.md").read_text(encoding="utf-8")
    if not re.search(
        r"(?m)^## \[(?:Unreleased|" + re.escape(version) + r")\]", changelog
    ):
        errors.append("CHANGELOG.md must contain the current version or Unreleased")
    if errors:
        raise ValueError("\n".join(errors))
    print("Version metadata is consistent.")


def sync(root: Path = ROOT, *, value: str | None = None) -> None:
    release = value if value is not None else _version_target(root)[0]
    if not RELEASE.fullmatch(release):
        raise ValueError(f"version must use X.Y.Z release form: {release!r}")
    version = release
    python = python_version(version)
    updates = {
        root / "Cargo.toml": replace_section(
            (root / "Cargo.toml").read_text(encoding="utf-8"),
            "workspace.package",
            version,
        ),
        root / "pyproject.toml": replace_section(
            (root / "pyproject.toml").read_text(encoding="utf-8"), "project", python
        ),
    }
    sdk_path = root / "packages/dokime/pyproject.toml"
    updates[sdk_path] = replace_section(
        sdk_path.read_text(encoding="utf-8"), "project", python
    )
    for relative in [
        "crates/dokime-gui/package.json",
        "crates/dokime-gui/src-tauri/tauri.conf.json",
    ]:
        path = root / relative
        text, count = re.subn(
            r'(?m)^(\s*"version"\s*:\s*)"[^"]*"',
            lambda m: m[1] + json.dumps(version),
            path.read_text(encoding="utf-8"),
            count=1,
        )
        if count != 1:
            raise ValueError(f"missing version in {relative}")
        updates[path] = text
    changed = []
    for path, text in updates.items():
        if path.read_text(encoding="utf-8") != text:
            path.write_text(text, encoding="utf-8")
            changed.append(str(path.relative_to(root)))
    subprocess.run(["cargo", "update", "--workspace"], cwd=root, check=True)
    subprocess.run(["uv", "lock"], cwd=root, check=True)
    if changed:
        print(f"Updated {len(changed)} version file(s) for {version}:")
        print("\n".join(f"- {path}" for path in changed))
    else:
        print(f"Version files are already stamped for {version}.")


def prepare(root: Path = ROOT) -> None:
    branch = subprocess.check_output(
        ["git", "branch", "--show-current"], cwd=root, text=True, encoding="utf-8"
    ).strip()
    if subprocess.check_output(
        ["git", "status", "--porcelain", "--untracked-files=no"],
        cwd=root,
        text=True,
        encoding="utf-8",
    ).strip():
        raise ValueError("prepare a candidate from clean tracked sources")
    candidate = (
        subprocess.check_output(
            [
                "git-cliff",
                "--config",
                "cliff.toml",
                "--bumped-version",
                "--bump",
                "patch",
            ],
            cwd=root,
            text=True,
            encoding="utf-8",
        )
        .strip()
        .removeprefix("v")
    )
    settings = _toml(root / "stkr.toml")
    hotfix = branch.startswith("fix/") and isinstance(
        settings.get("release", {}).get("hotfix"), dict
    )
    if not RELEASE.fullmatch(candidate) or (
        branch != f"release/{candidate}" and not hotfix
    ):
        raise ValueError("prepare requires release/<next-patch> or a declared hotfix")
    sync(root, value=candidate)
    subprocess.run(
        [
            "git-cliff",
            "--config",
            "cliff.toml",
            "--bump",
            "patch",
            "--tag",
            "v" + candidate,
            "--output",
            "CHANGELOG.md",
        ],
        cwd=root,
        check=True,
    )
    check(root)


def replace_section(text: str, section: str, version: str) -> str:
    pattern = r"(?ms)(^\[" + re.escape(section) + r"\]\r?\n)(.*?)(?=^\[|\Z)"

    def replace(match: re.Match[str]) -> str:
        body, count = re.subn(
            r"(?m)^(\s*)version\s*=.*$",
            lambda m: f'{m[1]}version = "{version}"',
            match[2],
            count=1,
        )
        if count != 1:
            raise ValueError(f"missing version key in [{section}]")
        return match[1] + body

    result, count = re.subn(pattern, replace, text, count=1)
    if count != 1:
        raise ValueError(f"missing [{section}]")
    return result


def run(root: Path, verb: str, *, value: str | None = None) -> None:
    if verb == "sync":
        sync(root, value=value)
        return
    if value is not None:
        raise ValueError("an explicit version is only valid for sync")
    if verb == "check":
        check(root)
    elif verb == "prepare":
        prepare(root)
    else:
        raise ValueError(f"unknown version operation: {verb}")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=["check", "sync", "prepare"])
    args = parser.parse_args()
    try:
        run(ROOT, args.command)
    except (OSError, ValueError, KeyError, subprocess.CalledProcessError) as error:
        print(error, file=sys.stderr)
        return 1
    return 0


# === Private Functions === #


def _toml(path: Path) -> dict:
    return tomllib.loads(path.read_text(encoding="utf-8"))


def _package_names(root: Path, manifest: dict) -> set[str]:
    return {
        _toml(root / member / "Cargo.toml")["package"]["name"]
        for member in manifest["workspace"]["members"]
        if (root / member / "Cargo.toml").exists()
    }


def _version_target(root: Path) -> tuple[str, str]:
    mode = os.environ.get("DOKIME_VERSION_MODE", "").strip()
    release = os.environ.get("DOKIME_VERSION_RELEASE", "").strip()
    if mode or release:
        if mode != "release" or not release:
            raise ValueError(
                "set both DOKIME_VERSION_MODE=release and DOKIME_VERSION_RELEASE=X.Y.Z"
            )
        return release, mode
    tag = (
        os.environ.get("GITHUB_REF_NAME", "").strip()
        if os.environ.get("GITHUB_REF_TYPE") == "tag"
        else ""
    )
    if tag:
        if not re.fullmatch(r"v\d+\.\d+\.\d+", tag):
            raise ValueError("GITHUB_REF_NAME must match vX.Y.Z")
        return tag[1:], "release"
    command = ["git-cliff"]
    latest_tag = False
    if (root / ".jj").exists():
        # Snapshot the selected workspace before checking whether its content
        # is the exact release, or an empty working child of that release.
        tag_template = 'if(tags, tags.map(|t| t.name()).join("\\n"), if(empty, parents.map(|p| p.tags().map(|t| t.name()).join("\\n")).join("\\n"), ""))'
        head_tags = subprocess.check_output(
            ["jj", "log", "-r", "@", "--no-graph", "-T", tag_template],
            cwd=root,
            text=True,
            encoding="utf-8",
        ).splitlines()
        for head_tag in head_tags:
            if re.fullmatch(r"v\d+\.\d+\.\d+", head_tag):
                return head_tag[1:], "release"
        ancestor_tags = subprocess.check_output(
            [
                "jj",
                "--ignore-working-copy",
                "log",
                "-r",
                "tags() & ::@",
                "--no-graph",
                "-T",
                'tags.map(|t| t.name()).join("\\n") ++ "\\n"',
            ],
            cwd=root,
            text=True,
            encoding="utf-8",
        ).splitlines()
        latest_tag = any(
            re.fullmatch(r"v\d+\.\d+\.\d+", value) for value in ancestor_tags
        )
        repository = subprocess.check_output(
            ["jj", "--ignore-working-copy", "git", "root"],
            cwd=root,
            text=True,
            encoding="utf-8",
        ).strip()
        command += ["--repository", repository]
    bumped = (
        subprocess.check_output(
            command + ["--bumped-version", "--bump", "patch"],
            cwd=root,
            text=True,
            encoding="utf-8",
        )
        .strip()
        .removeprefix("v")
    )
    current = _toml(root / "Cargo.toml")["workspace"]["package"]["version"]
    if not RELEASE.fullmatch(bumped):
        raise ValueError(f"git-cliff returned an invalid version: {bumped!r}")
    # An untagged development repository may already carry a higher baseline.
    version = (
        bumped
        if latest_tag
        else max([current, bumped], key=lambda v: tuple(int(p) for p in v.split(".")))
    )
    return version, "release"


# === Entry Point === #

if __name__ == "__main__":
    raise SystemExit(main())
