# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Build Portable Linux Wheels From Prepared Source Archives"""

# === Imports === #

# Standard Library Imports
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import shutil
import subprocess
import sys
import tomllib

# Third Party Imports

# Local Imports

# Type Checking Imports


# === Public Exports === #

# === Constants === #
IMAGE = "quay.io/pypa/manylinux_2_28_x86_64@sha256:4131ddd7acb9c6c966af02fe7e71edc51e078c2d41f475966697676430bfa2ae"
COMPATIBILITY = "manylinux_2_28"
RECEIPT = "wheel-build.json"
PACKAGES = (
    "dbus-devel-1:1.12.8-28.el8_10.x86_64",
    "cmake-filesystem-0:3.26.5-2.el8.x86_64",
    "xml-common-0:0.6.3-50.el8.noarch",
)

# === Types === #

# === Classes === #

# === Public Functions === #


def build(archive: Path, directory: Path, root: Path) -> None:
    if platform.machine() != "x86_64" or shutil.which("docker") is None:
        raise SystemExit("Linux release wheels require x86_64 and Docker.")
    tools = tomllib.loads((root / "mise.toml").read_text())["tools"]
    cache = root / ".cache/manylinux"
    target = root / "target/python/manylinux"
    cargo = Path(os.environ.get("CARGO_HOME", Path.home() / ".cargo"))
    sysroot = Path(_output(["rustc", "--print", "sysroot"]))
    uv = Path(shutil.which("uv") or "").resolve()
    mounts = {
        archive.resolve(): (f"/input/{archive.name}", True),
        directory.resolve(): ("/output", False),
        cache: ("/cache", False),
        target: ("/target", False),
        sysroot: ("/toolchain", True),
        Path(sys.base_prefix).resolve(): ("/python", True),
        uv: ("/usr/local/bin/uv", True),
        Path(__file__).resolve(): ("/builder.py", True),
    }
    cache.mkdir(parents=True, exist_ok=True)
    target.mkdir(parents=True, exist_ok=True)
    for name in ("registry", "git"):
        source = (cargo / name).resolve()
        if not source.is_dir():
            raise SystemExit(f"Missing fetched Cargo inputs: {source}")
        (cache / "cargo" / name).mkdir(parents=True, exist_ok=True)
        mounts[source] = (f"/cache/cargo/{name}", True)
    command = ["docker", "run", "--rm", "--platform", "linux/amd64"]
    for source, (destination, readonly) in mounts.items():
        mount = f"type=bind,source={source},target={destination}"
        command.extend(["--mount", mount + (",readonly" if readonly else "")])
    command.extend(
        [
            IMAGE,
            "/python/bin/python3.13",
            "/builder.py",
            archive.name,
            str(os.getuid()),
            str(os.getgid()),
            str(tools["python"]),
            str(tools["uv"]),
            str(tools["rust"]["version"]),
            _maturin(root),
        ]
    )
    subprocess.run(command, check=True)


def verify(
    receipt: dict, wheel_hash: str, archive_hash: str, rustc: str, root: Path
) -> None:
    tools = tomllib.loads((root / "mise.toml").read_text())["tools"]
    if (
        receipt.get("version") != 1
        or receipt.get("image") != IMAGE
        or receipt.get("compatibility") != COMPATIBILITY
        or receipt.get("wheel_sha256") != wheel_hash
        or receipt.get("sdist_sha256") != archive_hash
        or receipt.get("rustc") != rustc
        or receipt.get("python") != f"Python {tools['python']}"
        or str(receipt.get("uv", "")).split()[:2] != ["uv", str(tools["uv"])]
        or receipt.get("maturin") != _maturin(root).replace("==", " ")
        or not isinstance(receipt.get("packages"), list)
        or not receipt["packages"]
        or not set(PACKAGES) <= set(receipt["packages"])
    ):
        raise ValueError("Linux wheel build provenance differs from its packages")


# === Private Functions === #


def _output(command: list[str]) -> str:
    return subprocess.check_output(command, text=True).strip()


def _digest(path: Path) -> str:
    with path.open("rb") as content:
        return hashlib.file_digest(content, "sha256").hexdigest()


def _maturin(root: Path) -> str:
    path = root / "crates/dokime-python/pyproject.toml"
    requires = tomllib.loads(path.read_text())["build-system"]["requires"]
    if len(requires) != 1 or not re.fullmatch(r"maturin==\d+\.\d+\.\d+", requires[0]):
        raise ValueError("The SDK build backend must pin one exact Maturin version")
    return requires[0]


def _container() -> None:
    archive_name, uid, gid, python_version, uv_version, rust_version, maturin = (
        sys.argv[1:]
    )
    # Install Native Libraries Inside The Pinned GLIBC Environment.
    query = [
        "rpm",
        "-qa",
        "--qf",
        "%{NAME}-%{EPOCHNUM}:%{VERSION}-%{RELEASE}.%{ARCH}\n",
    ]
    original = set(_output(query).splitlines())
    subprocess.run(
        ["dnf", "install", "-y", "--setopt=install_weak_deps=False", *PACKAGES],
        check=True,
    )
    packages = set(_output(query).splitlines())
    if packages != original | set(PACKAGES):
        raise ValueError("Container native package resolution differs from its pins")
    os.setgroups([])
    os.setgid(int(gid))
    os.setuid(int(uid))
    os.environ.update(
        {
            "HOME": "/cache/home",
            "PATH": "/toolchain/bin:" + os.environ["PATH"],
            "CARGO_HOME": "/cache/cargo",
            "CARGO_TARGET_DIR": "/target",
            "UV_CACHE_DIR": "/cache/uv",
            "PYO3_NO_PYTHON": "1",
        }
    )
    Path("/cache/home").mkdir(exist_ok=True)
    if _output(["uv", "--version"]).split()[:2] != ["uv", uv_version]:
        raise ValueError("Container uv differs from the repository pin")
    rustc = _output(["rustc", "--version"])
    if not rustc.startswith(f"rustc {rust_version} "):
        raise ValueError("Container Rust differs from the repository pin")
    environment = Path("/cache/environment")
    subprocess.run(
        [
            "uv",
            "venv",
            "--clear",
            "--python",
            "/python/bin/python3.13",
            str(environment),
        ],
        check=True,
    )
    python = str(environment / "bin/python")
    if _output([python, "--version"]) != f"Python {python_version}":
        raise ValueError("Container Python differs from the repository pin")
    subprocess.run(["uv", "pip", "install", "--python", python, maturin], check=True)
    # Build The Exact Prepared Archive With Native Cargo Locks And Auditing.
    archive = Path("/input") / archive_name
    subprocess.run(
        [
            "uv",
            "build",
            str(archive),
            "--wheel",
            "--python",
            python,
            "--no-build-isolation",
            "--out-dir",
            "/output",
            "--config-setting",
            "maturin.build-args=--locked --offline --compatibility manylinux_2_28 --auditwheel repair",
        ],
        check=True,
    )
    wheels = list(Path("/output").glob("dokime-*.whl"))
    if len(wheels) != 1:
        raise ValueError("Expected one newly built portable Linux wheel")
    receipt = {
        "version": 1,
        "image": IMAGE,
        "compatibility": COMPATIBILITY,
        "rustc": rustc,
        "uv": _output(["uv", "--version"]),
        "python": _output([python, "--version"]),
        "maturin": _output([str(environment / "bin/maturin"), "--version"]),
        "packages": sorted(packages),
        "wheel_sha256": _digest(wheels[0]),
        "sdist_sha256": _digest(archive),
    }
    Path("/output", RECEIPT).write_text(json.dumps(receipt, indent=2) + "\n")


# === Entry Point === #

if __name__ == "__main__":
    _container()
