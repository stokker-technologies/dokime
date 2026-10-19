# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Verify Candidate Tutorial Provenance And Artifact-Only Packaging"""

# === Imports === #

# Standard Library Imports
import hashlib
import shutil
import zipfile
from pathlib import Path

# Third Party Imports
import pytest

# Local Imports
from dokime_tools.package import examples

# Type Checking Imports

# === Public Exports === #

# === Constants === #

WHEEL = "dokime_client-0.1.0-py3-none-any.whl"

# === Types === #

# === Classes === #

# === Public Functions === #


def test_candidate_archive(tmp_path: Path) -> None:
    root, wheel = _fixture(tmp_path)
    original = wheel.read_bytes()
    source = root / "packages/dokime-client/examples/remote"
    (source / ".env").write_text("PRIVATE_KEY=secret", encoding="utf-8")
    (source / "python/unrelated.txt").write_text("private", encoding="utf-8")
    destination = root / "target/build/dev/any/python/examples/remote"
    staged = examples.stage(root, destination, wheel=wheel)
    expected = {*examples.FILES, f"wheels/{WHEEL}"}
    assert {path.relative_to(destination).as_posix() for path in staged} == expected

    # Package Reads Only Frozen Build Inputs, Including The Original Wheel Bytes
    shutil.rmtree(source)
    (destination / "local.key").write_bytes(b"private")
    examples.package(root, "dev")
    output = root / "target/package/dev/any/python/dokime-remote-examples-0.1.0.zip"
    with zipfile.ZipFile(output) as archive:
        assert set(archive.namelist()) == {f"remote/{relative}" for relative in expected}
        assert archive.read(f"remote/wheels/{WHEEL}") == original
        assert archive.read("remote/python/uv.lock") == (destination / "python/uv.lock").read_bytes()
        assert archive.getinfo("remote/cli/control-test.sh").external_attr >> 16 == 0o100755


@pytest.mark.parametrize("fault", ["wheel", "hash", "version", "source", "pin", "python"])
def test_candidate_refusal(tmp_path: Path, fault: str) -> None:
    root, wheel = _fixture(tmp_path)
    project = root / "packages/dokime-client/examples/remote/python"
    lock = project / "uv.lock"
    manifest = project / "pyproject.toml"
    if fault == "wheel":
        wheel.write_bytes(b"different bytes at the same wheel version")
    elif fault == "hash":
        lock.write_text(lock.read_text().replace(hashlib.sha256(wheel.read_bytes()).hexdigest(), "0" * 64))
    elif fault == "version":
        lock.write_text(lock.read_text().replace('version = "0.1.0"', 'version = "0.2.0"'))
    elif fault == "source":
        manifest.write_text(manifest.read_text().replace("../wheels/", "../../outside/"))
    elif fault == "pin":
        manifest.write_text(manifest.read_text().replace("dokime-client==0.1.0", "dokime-client==0.2.0"))
    else:
        manifest.write_text(manifest.read_text().replace(">=3.13,<3.16", ">=3.14,<3.16"))

    destination = root / "target/build/dev/any/python/examples/remote"
    destination.mkdir(parents=True)
    retained = destination / "retained"
    retained.write_bytes(b"previous valid candidate")
    with pytest.raises(ValueError):
        examples.stage(root, destination, wheel=wheel)
    assert retained.read_bytes() == b"previous valid candidate"


@pytest.mark.parametrize("fault", ["wheel", "lock", "distribution"])
def test_built_candidate_refused(tmp_path: Path, fault: str) -> None:
    root, wheel = _fixture(tmp_path)
    destination = root / "target/build/dev/any/python/examples/remote"
    examples.stage(root, destination, wheel=wheel)
    examples.package(root, "dev")
    archive = root / "target/package/dev/any/python/dokime-remote-examples-0.1.0.zip"
    previous = archive.read_bytes()
    if fault == "wheel":
        (destination / "wheels" / WHEEL).write_bytes(b"tampered built candidate")
    elif fault == "distribution":
        wheel.write_bytes(b"tampered built distribution")
    else:
        lock = destination / "python/uv.lock"
        lock.write_text(lock.read_text().replace(hashlib.sha256(wheel.read_bytes()).hexdigest(), "0" * 64))
    with pytest.raises(ValueError):
        examples.package(root, "dev")
    assert archive.read_bytes() == previous


@pytest.mark.parametrize("subject", ["wheel", "script", "directory"])
def test_link_refusal(tmp_path: Path, subject: str) -> None:
    root, wheel = _fixture(tmp_path)
    source = root / "packages/dokime-client/examples/remote"
    path = wheel if subject == "wheel" else source / "python/control-test.py"
    if subject == "directory":
        directory = source / "python"
        external = tmp_path / "external"
        directory.rename(external)
        directory.symlink_to(external, target_is_directory=True)
    else:
        external = tmp_path / "external"
        path.rename(external)
        path.symlink_to(external)
    with pytest.raises(ValueError, match="regular file"):
        examples.stage(root, root / "target/build/dev/any/python/examples/remote", wheel=wheel)


# === Private Functions === #


def _fixture(tmp_path: Path) -> tuple[Path, Path]:
    root = tmp_path / "repository"
    package = root / "packages/dokime-client"
    source = package / "examples/remote"
    for relative in examples.FILES:
        path = package / relative if relative == "LICENSE-APACHE" else source / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(f"Public Fixture: {relative}\n", encoding="utf-8")
        if path.suffix == ".sh":
            path.chmod(0o755)
    workspace = Path(__file__).resolve().parents[5]
    shutil.copyfile(workspace / "stkr.toml", root / "stkr.toml")
    wheel = root / "target/build/dev/any/python" / WHEEL
    wheel.parent.mkdir(parents=True)
    wheel.write_bytes(b"fixture candidate wheel bytes")
    digest = hashlib.sha256(wheel.read_bytes()).hexdigest()
    (source / "python/pyproject.toml").write_text(
        '[project]\nname = "dokime-remote-examples"\nversion = "0.1.0"\nrequires-python = ">=3.13,<3.16"\n'
        'dependencies = ["dokime-client==0.1.0"]\n[tool.uv.sources]\n'
        f'dokime-client = {{ path = "../wheels/{WHEEL}" }}\n',
        encoding="utf-8",
    )
    (source / "python/uv.lock").write_text(
        'version = 1\nrequires-python = ">=3.13,<3.16"\n[[package]]\nname = "dokime-client"\nversion = "0.1.0"\n'
        f'source = {{ path = "../wheels/{WHEEL}" }}\n'
        f'wheels = [{{ filename = "{WHEEL}", hash = "sha256:{digest}" }}]\n',
        encoding="utf-8",
    )
    return root, wheel


# === Entry Point === #
