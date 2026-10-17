# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Locate Contract Exports From A Selected Cargo Build"""

# === Imports === #

# Standard Library Imports
import argparse
import hashlib
import io
import json
import os
import shutil
import subprocess
import tarfile
import tempfile
from pathlib import Path

# Third Party Imports

# Local Imports

# Type Checking Imports

# === Public Exports === #

# === Constants === #
_MAX_ARCHIVE_BYTES = 64 * 1024 * 1024
_MAX_MEMBER_BYTES = 8 * 1024 * 1024
_MAX_ASSETS = 4096
CORE_MEMBERS = frozenset(
    {"manifest.txt", "revision.txt", "contracts.json", "descriptors.bin"}
)

# === Types === #

# === Classes === #

# === Public Functions === #


def locate(messages: Path) -> Path:
    artifacts: set[str] = set()
    scripts: dict[str, set[Path]] = {}
    finished: list[bool] = []
    for line in messages.read_text(encoding="utf-8").splitlines():
        event = json.loads(line)
        reason = event.get("reason")
        if reason == "compiler-artifact":
            target = event["target"]
            # The server build script exports the operation contracts it serves.
            if target["name"] == "dokime-server" and "bin" in target["kind"]:
                artifacts.add(event["package_id"])
        elif reason == "build-script-executed":
            package = event["package_id"]
            output = Path(event["out_dir"])
            scripts.setdefault(package, set()).add(output)
        elif reason == "build-finished":
            finished.append(event["success"])
    if finished != [True] or len(artifacts) != 1:
        raise ValueError(
            "expected one dokime-server contract producer from a successful Cargo build"
        )
    outputs = scripts.get(next(iter(artifacts)), set())
    if not outputs:
        raise ValueError("selected Dokime build has no build-script output")
    if len(outputs) != 1:
        raise ValueError("selected Dokime build has multiple build variants")
    output = next(iter(outputs))
    root = output / "grpc/operations"
    descriptor = root / "descriptors.bin"
    manifest = root / "manifest.txt"
    if not descriptor.is_file() or not manifest.is_file():
        raise ValueError("selected Dokime contract build did not export contracts")
    names = manifest.read_text(encoding="utf-8").splitlines()
    if not {"descriptors.bin", "contracts.json"}.issubset(names) or len(names) != len(
        set(names)
    ):
        raise ValueError("invalid contract export manifest")
    for name in names:
        relative = Path(name)
        if (
            not name
            or "\\" in name
            or ":" in name
            or relative.is_absolute()
            or ".." in relative.parts
            or relative.as_posix() != name
            or (
                name not in {"descriptors.bin", "contracts.json"}
                and relative.suffix != ".proto"
            )
        ):
            raise ValueError(f"invalid contract export path: {name}")
        if not (root / relative).is_file():
            raise ValueError(f"missing selected contract export: {name}")
    return root


def prepare(
    messages: Path, *, profile: str, root: Path, cargo_config: Path | None = None
) -> Path:
    if cargo_config is not None:
        cargo_config = cargo_config.resolve(strict=True)
        if not cargo_config.is_file():
            raise ValueError("Cargo configuration must be a file")
    messages.parent.mkdir(parents=True, exist_ok=True)
    temporary = messages.with_suffix(".pending")
    command = ["cargo"]
    if cargo_config is not None:
        command.extend(["--config", str(cargo_config)])
    command.extend(
        [
            "build",
            "--locked",
            "-p",
            "dokime-contract",
            "--bin",
            "dokime-contract-export",
            "--features",
            "descriptors",
            "--message-format=json-render-diagnostics",
        ]
    )
    if profile == "release":
        command.append("--release")
    try:
        with temporary.open("w", encoding="utf-8") as output:
            subprocess.run(command, cwd=root, stdout=output, check=True)
        events = [
            json.loads(line)
            for line in temporary.read_text(encoding="utf-8").splitlines()
        ]
        if [
            event.get("success")
            for event in events
            if event.get("reason") == "build-finished"
        ] != [True]:
            raise ValueError("contract producer build did not complete successfully")
        producers = [
            event
            for event in events
            if event.get("reason") == "compiler-artifact"
            and event["target"]["name"] == "dokime-contract-export"
            and "bin" in event["target"]["kind"]
        ]
        if len(producers) != 1 or not producers[0].get("executable"):
            raise ValueError("selected Cargo build has no unique contract exporter")
        subprocess.run([producers[0]["executable"]], cwd=root, check=True)
        temporary.replace(messages)
        return locate(messages)
    finally:
        temporary.unlink(missing_ok=True)


def archive(
    messages: Path | None,
    destination: Path,
    revision: str,
    *,
    root: Path,
    export: Path | None = None,
) -> None:
    if len(revision) != 40 or any(
        character not in "0123456789abcdef" for character in revision
    ):
        raise ValueError("contract archive requires the exact source commit")
    if export is not None:
        exported = restored(export, revision)
    elif messages is not None:
        exported = locate(messages)
    else:
        raise ValueError("an exact restored export or selected Cargo build is required")
    names = (exported / "manifest.txt").read_text(encoding="utf-8").splitlines()
    destination.parent.mkdir(parents=True, exist_ok=True)
    temporary = destination.with_suffix(".pending")
    try:
        with tarfile.open(temporary, "w", format=tarfile.USTAR_FORMAT) as output:
            contents = [
                (name, (exported / name).read_bytes()) for name in sorted(names)
            ]
            contents.extend(
                [
                    ("manifest.txt", (exported / "manifest.txt").read_bytes()),
                    ("revision.txt", (revision + "\n").encode("ascii")),
                ]
            )
            for name, data in contents:
                member = tarfile.TarInfo(name)
                member.size = len(data)
                member.mode = 0o644
                member.mtime = 0
                output.addfile(member, io.BytesIO(data))
        temporary.replace(destination)
        with destination.open("rb") as source:
            digest = hashlib.file_digest(source, "sha256").hexdigest()
        destination.with_suffix(destination.suffix + ".sha256").write_text(
            f"{digest}  {destination.name}\n", encoding="ascii"
        )
    finally:
        temporary.unlink(missing_ok=True)


def members(archive_path: Path) -> dict[str, bytes]:
    """Validated Portable Contract Archive Members Keyed By Canonical Name."""
    if archive_path.stat().st_size > _MAX_ARCHIVE_BYTES:
        raise ValueError("contract archive exceeds its bounded metadata limit")
    contents: dict[str, bytes] = {}
    with tarfile.open(archive_path, "r:") as selected:
        for index, member in enumerate(selected):
            name = member.name
            relative = Path(name)
            if (
                index >= _MAX_ASSETS
                or member.size > _MAX_MEMBER_BYTES
                or not member.isfile()
                or name in contents
                or not name
                or "\\" in name
                or ":" in name
                or relative.is_absolute()
                or ".." in relative.parts
                or relative.as_posix() != name
            ):
                raise ValueError("contract archive contains an invalid member")
            if name not in CORE_MEMBERS and relative.suffix != ".proto":
                raise ValueError("contract archive contains an unexpected asset")
            source = selected.extractfile(member)
            if source is None:
                raise ValueError("contract archive member is unreadable")
            with source:
                data = source.read(member.size + 1)
            if len(data) != member.size:
                raise ValueError("contract archive member length differs")
            contents[name] = data
    names = contents.get("manifest.txt", b"").decode("utf-8").splitlines()
    if (
        len(names) != len(set(names))
        or set(contents) != set(names) | {"manifest.txt", "revision.txt"}
        or not {"contracts.json", "descriptors.bin"}.issubset(names)
    ):
        raise ValueError("contract archive content differs from its manifest")
    return contents


def restore(archive_path: Path, sha256: str, revision: str, destination: Path) -> Path:
    if len(sha256) != 64 or any(
        character not in "0123456789abcdef" for character in sha256
    ):
        raise ValueError("restored contracts require the exact artifact SHA-256")
    if len(revision) != 40 or any(
        character not in "0123456789abcdef" for character in revision
    ):
        raise ValueError("restored contracts require the exact source commit")
    contents = members(archive_path)
    with archive_path.open("rb") as source:
        if hashlib.file_digest(source, "sha256").hexdigest() != sha256:
            raise ValueError("contract archive differs from the selected build catalog")
    if contents["revision.txt"] != (revision + "\n").encode("ascii"):
        raise ValueError("contract archive belongs to another source commit")
    destination.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(
        prefix="contracts-restore-", dir=destination.parent
    ) as directory:
        staged = Path(directory) / "export"
        staged.mkdir()
        for name, data in contents.items():
            target = staged / name
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(data)
        receipt = {
            "schema": 1,
            "revision": revision,
            "archive_sha256": sha256,
            "files": {
                name: hashlib.sha256(data).hexdigest()
                for name, data in contents.items()
            },
        }
        (staged / ".restored.json").write_text(
            json.dumps(receipt, sort_keys=True) + "\n", encoding="utf-8"
        )
        if destination.is_symlink():
            raise ValueError("refusing to replace a linked contract directory")
        if destination.exists():
            if not (destination / ".restored.json").is_file():
                raise ValueError("refusing to replace an unowned contract directory")
            shutil.rmtree(destination)
        staged.replace(destination)
    return destination


def restored(export: Path, revision: str) -> Path:
    if export.is_symlink():
        raise ValueError("restored contract root must not be a symlink")
    receipt = json.loads((export / ".restored.json").read_text(encoding="utf-8"))
    if receipt.get("schema") != 1 or receipt.get("revision") != revision:
        raise ValueError(
            "restored contracts do not belong to the selected source commit"
        )
    files = receipt.get("files")
    if not isinstance(files, dict) or not {
        "manifest.txt",
        "revision.txt",
        "contracts.json",
        "descriptors.bin",
    }.issubset(files):
        raise ValueError("restored contracts have no complete provenance")
    for name, expected in files.items():
        relative = Path(name)
        if (
            not name
            or "\\" in name
            or ":" in name
            or relative.is_absolute()
            or ".." in relative.parts
            or relative.as_posix() != name
        ):
            raise ValueError("restored contract provenance contains an invalid path")
        target = export / relative
        ancestors = [target, *target.parents]
        linked = any(
            path.is_symlink()
            for path in ancestors
            if path == export or export in path.parents
        )
        if (
            linked
            or not target.is_file()
            or hashlib.sha256(target.read_bytes()).hexdigest() != expected
        ):
            raise ValueError(
                "restored contracts differ from the verified build artifact"
            )
    names = (export / "manifest.txt").read_text(encoding="utf-8").splitlines()
    if set(files) != set(names) | {"manifest.txt", "revision.txt"}:
        raise ValueError("restored contract provenance differs from its manifest")
    if (export / "revision.txt").read_text(encoding="ascii") != revision + "\n":
        raise ValueError("restored contract revision differs from its provenance")
    return export


def main() -> None:
    root = Path(__file__).resolve().parents[5]
    parser = argparse.ArgumentParser(description=__doc__)
    subcommands = parser.add_subparsers(dest="command", required=True)
    selected = subcommands.add_parser("locate")
    selected.add_argument("messages", type=Path)
    build = subcommands.add_parser("prepare")
    build.add_argument("--profile", choices=("dev", "release"), default="dev")
    build.add_argument("--cargo-config", type=Path)
    build.add_argument(
        "--build-json", type=Path, default=os.environ.get("DOKIME_CONTRACT_BUILD_JSON")
    )
    build.add_argument(
        "--export", type=Path, default=os.environ.get("DOKIME_CONTRACT_EXPORT")
    )
    build.add_argument("--revision", default=os.environ.get("GITHUB_SHA"))
    package = subcommands.add_parser("archive")
    package.add_argument(
        "--build-json", type=Path, default=os.environ.get("DOKIME_CONTRACT_BUILD_JSON")
    )
    package.add_argument(
        "--output", type=Path, default=root / "target/contracts/dokime-contract.tar"
    )
    package.add_argument("--revision", default=os.environ.get("GITHUB_SHA"))
    package.add_argument(
        "--export", type=Path, default=os.environ.get("DOKIME_CONTRACT_EXPORT")
    )
    consume = subcommands.add_parser("restore")
    consume.add_argument("--archive", type=Path, required=True)
    consume.add_argument("--sha256", required=True)
    consume.add_argument("--revision", required=True)
    consume.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    if args.command == "prepare":
        if args.export is not None:
            revision = args.revision or _revision(root)
            print(restored(args.export, revision))
        else:
            if args.build_json is None:
                parser.error("--build-json must select the contract producer")
            print(
                prepare(
                    args.build_json,
                    profile=args.profile,
                    root=root,
                    cargo_config=args.cargo_config,
                )
            )
    elif args.command == "restore":
        print(restore(args.archive, args.sha256, args.revision, args.output))
    elif args.command == "archive":
        if args.build_json is None and args.export is None:
            parser.error("--build-json or --export must select the producer")
        revision = args.revision or _revision(root)
        archive(args.build_json, args.output, revision, root=root, export=args.export)
    else:
        print(locate(args.messages))


# === Private Functions === #


def _revision(root: Path) -> str:
    return subprocess.check_output(
        ["git", "rev-parse", "HEAD"], cwd=root, text=True
    ).strip()


# === Entry Point === #

if __name__ == "__main__":
    main()
