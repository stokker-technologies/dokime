# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Keep Harmos Tool Preparation Exact And Offline Across Source Modes"""

# === Imports === #

# Standard Library Imports
import hashlib
import json
import os
from pathlib import Path

# Third Party Imports
import pytest

# Local Imports
from dokime_tools.ci import bootstrap
from dokime_tools.prepare import bindings

# Type Checking Imports

# === Public Exports === #

# === Constants === #

REVISION = "1" * 40

# === Types === #

# === Classes === #

# === Public Functions === #


@pytest.fixture(autouse=True)
def processes(monkeypatch: pytest.MonkeyPatch):
    calls = []
    source = ["local-source"]
    version = ["0.1.0"]
    for name in ("CI", "GITHUB_ACTIONS", "DOKIME_CI_ROLE", "GITHUB_PATH"):
        monkeypatch.delenv(name, raising=False)

    def refuse(*args, **kwargs):
        pytest.fail(f"unexpected external process: {args}")

    def output(command, **kwargs):
        if command == ["mise", "exec", "--", "rustc", "-vV"]:
            return "rustc offline fixture"
        if len(command) == 2 and command[1] == "--version":
            executable = Path(command[0])
            assert executable.read_bytes() == b"offline tool"
            return f"cargo harmos {version[0]}"
        return refuse(command)

    def install(command):
        assert command[:6] == ["mise", "exec", "--", "cargo", "install", "--debug"]
        assert "--locked" in command
        assert command[-1] == "cargo-harmos"
        calls.append(command)
        staged = Path(command[command.index("--root") + 1])
        binary = staged / "bin" / ("cargo-harmos.exe" if os.name == "nt" else "cargo-harmos")
        binary.parent.mkdir()
        binary.write_bytes(b"offline tool")

    def release(command, **kwargs):
        assert command[:2] == ["mise", "install-into"]
        assert command[2].startswith("github:stokker-technologies/harmos[asset_pattern=cargo-harmos-")
        assert kwargs["cwd"] == Path(command[3]).parent
        assert "MISE_ENABLE_TOOLS" not in kwargs["env"]
        calls.append(command)
        version[0] = command[2].rsplit("@", 1)[1]
        staged = Path(command[3])
        binary = staged / "bin" / ("cargo-harmos.exe" if os.name == "nt" else "cargo-harmos")
        binary.parent.mkdir(parents=True)
        binary.write_bytes(b"offline tool")

    def retrieve(url, path):
        assert url.endswith("/SHA256SUMS")
        selected = url.rsplit("/", 2)[1].removeprefix("v")
        digest = hashlib.sha256(b"offline tool").hexdigest()
        Path(path).write_text(
            "".join(
                f"{digest}  cargo-harmos-{selected}-{target}\n" for target in ("x86_64-pc-windows-msvc.exe", "x86_64-unknown-linux-gnu")
            )
        )

    def run(command, **kwargs):
        return release(command, **kwargs) if command[:2] == ["mise", "install-into"] else refuse(command, **kwargs)

    monkeypatch.setattr(bootstrap.urllib.request, "urlretrieve", retrieve)
    monkeypatch.setattr(bootstrap.subprocess, "run", run)
    monkeypatch.setattr(bootstrap.subprocess, "Popen", refuse)
    monkeypatch.setattr(bootstrap.subprocess, "check_output", output)
    monkeypatch.setattr(bootstrap, "_run", install)
    monkeypatch.setattr(bootstrap, "_override_source", lambda _path: source[0])
    return calls, source


def test_public_source_selection(tmp_path: Path, monkeypatch: pytest.MonkeyPatch, capsys, processes) -> None:
    _manifest(tmp_path, f'{{ git = "{bootstrap.HARMOS_REPOSITORY}", rev = "{REVISION}" }}')
    assert bootstrap.harmos_pin(tmp_path) == {"git": bootstrap.HARMOS_REPOSITORY, "rev": REVISION, "package": "cargo-harmos"}
    monkeypatch.chdir(tmp_path)
    assert bootstrap.main(["--harmos-key"]) == 0
    assert capsys.readouterr().out == REVISION + "\n"

    _manifest(tmp_path, f'{{ git = "{bootstrap.HARMOS_REPOSITORY}", branch = "release/next" }}')
    source = f"git+{bootstrap.HARMOS_REPOSITORY}?branch=release%2Fnext#{REVISION}"
    _lock(tmp_path, source=source)
    assert bootstrap.harmos_pin(tmp_path)["rev"] == REVISION
    assert bootstrap.harmos_pin(tmp_path)["branch"] == "release/next"

    _manifest(tmp_path, '"=0.1.0"')
    _lock(tmp_path)
    assert bootstrap.harmos_pin(tmp_path) == {"registry": "crates-io", "version": "0.1.0", "package": "cargo-harmos"}
    assert bootstrap.main(["--harmos-key"]) == 0
    assert capsys.readouterr().out == "crates-io-0.1.0\n"
    assert bootstrap.main(["--harmos-key", "--cargo-config", str(tmp_path / "unused-config")]) == 0
    assert capsys.readouterr().out == "crates-io-0.1.0\n"
    assert processes[0] == []


@pytest.mark.parametrize("fault", ["missing", "ambiguous", "foreign", "path", "version", "mixed", "absent", "range", "obsolete", "git"])
def test_source_refusal(tmp_path: Path, fault: str, processes) -> None:
    _manifest(tmp_path, '"=0.1.0"')
    _lock(tmp_path)
    manifest = tmp_path / "Cargo.toml"
    lock = tmp_path / "Cargo.lock"
    if fault == "missing":
        lock.unlink()
    elif fault == "ambiguous":
        lock.write_text(lock.read_text() + '\n[[package]]\nname = "harmos"\nversion = "0.1.0"\n')
    elif fault == "foreign":
        _lock(tmp_path, source="registry+https://private.invalid/index")
    elif fault == "path":
        lock.write_text(lock.read_text().replace(f'source = "{bootstrap.CRATES_SOURCE}"\n', ""))
    elif fault == "version":
        _lock(tmp_path, version="0.1.1")
    elif fault == "mixed":
        manifest.write_text(
            manifest.read_text().replace(
                'harmos-sdk = "=0.1.0"', f'harmos-sdk = {{ git = "{bootstrap.HARMOS_REPOSITORY}", rev = "{REVISION}" }}'
            )
        )
    elif fault == "absent":
        manifest.write_text(manifest.read_text().replace('harmos = "=0.1.0"\n', ""))
    elif fault == "range":
        _manifest(tmp_path, '"0.1"')
    elif fault == "obsolete":
        manifest.write_text(manifest.read_text() + 'harmos-contract = "0.1"\n')
    else:
        _manifest(tmp_path, f'{{ git = "{bootstrap.HARMOS_REPOSITORY}", rev = "{REVISION}" }}')
        manifest.write_text(
            manifest.read_text().replace(
                f'harmos-sdk = {{ git = "{bootstrap.HARMOS_REPOSITORY}", rev = "{REVISION}" }}', 'harmos-sdk = "=0.1.0"'
            )
        )
    with pytest.raises((ValueError, FileNotFoundError)):
        bootstrap.prepare_harmos(tmp_path, allow_install=True)
    assert processes[0] == []


def test_prepared_artifact(tmp_path: Path, monkeypatch: pytest.MonkeyPatch, processes) -> None:
    calls, _source = processes
    _manifest(tmp_path, f'{{ git = "{bootstrap.HARMOS_REPOSITORY}", rev = "{REVISION}" }}')
    binary = bootstrap.prepare_harmos(tmp_path, allow_install=True)
    assert calls[-1][6:10] == ["--git", bootstrap.HARMOS_REPOSITORY, "--rev", REVISION]

    _manifest(tmp_path, '{ version = "=0.1.0", default-features = false }')
    _lock(tmp_path)
    with pytest.raises(ValueError, match="consumers never rebuild"):
        bootstrap.prepare_harmos(tmp_path, allow_install=False)
    assert len(calls) == 1
    github_path = tmp_path / "github-path"
    monkeypatch.setenv("GITHUB_PATH", str(github_path))
    assert bootstrap.prepare_harmos(tmp_path, allow_install=True) == binary
    assert calls[-1][:2] == ["mise", "install-into"]
    assert calls[-1][2].endswith("]@0.1.0")
    receipt = binary.parent.parent / "receipt.json"
    value = json.loads(receipt.read_text())
    assert value["identity"]["registry"] == "crates-io"
    assert value["identity"]["version"] == "0.1.0"
    assert "rustc" not in value["identity"] and "profile" not in value["identity"]
    assert value["published_sha256"] == value["sha256"]
    assert value["version"] == "cargo harmos 0.1.0"
    assert value["sha256"] == hashlib.sha256(binary.read_bytes()).hexdigest()
    assert github_path.read_text().splitlines() == [str(binary.parent)]
    assert bootstrap.prepare_harmos(tmp_path, allow_install=False) == binary
    assert len(calls) == 2

    binary.write_bytes(b"tampered")
    with pytest.raises(ValueError, match="consumers never rebuild"):
        bootstrap.prepare_harmos(tmp_path, allow_install=False)
    binary.write_bytes(b"offline tool")
    value["identity"]["profile"] = "release"
    receipt.write_text(json.dumps(value))
    with pytest.raises(ValueError, match="consumers never rebuild"):
        bootstrap.prepare_harmos(tmp_path, allow_install=False)
    del value["identity"]["profile"]
    receipt.write_text(json.dumps(value))

    _manifest(tmp_path, '"=0.1.1"')
    _lock(tmp_path, version="0.1.1")
    with pytest.raises(ValueError, match="consumers never rebuild"):
        bootstrap.prepare_harmos(tmp_path, allow_install=False)
    assert len(calls) == 2
    assert bootstrap.prepare_harmos(tmp_path, allow_install=True) == binary
    assert calls[-1][:2] == ["mise", "install-into"]
    assert calls[-1][2].endswith("]@0.1.1")
    assert len(calls) == 3


@pytest.mark.parametrize("fault", ["checksum", "version", "manifest", "duplicate"])
def test_release_refusal(tmp_path: Path, monkeypatch: pytest.MonkeyPatch, fault: str, processes) -> None:
    _manifest(tmp_path, '"=0.1.0"')
    _lock(tmp_path)
    retrieve = bootstrap.urllib.request.urlretrieve
    output = bootstrap.subprocess.check_output

    def checksums(url, path):
        retrieve(url, path)
        manifest = Path(path)
        text = manifest.read_text()
        if fault == "checksum":
            text = text.replace(hashlib.sha256(b"offline tool").hexdigest(), "0" * 64)
        elif fault == "manifest":
            text = "not a checksum manifest"
        elif fault == "duplicate":
            text += text
        manifest.write_text(text)

    def version(command, **kwargs):
        if len(command) == 2 and command[1] == "--version" and fault == "version":
            return "cargo harmos 0.0.0"
        if command == ["mise", "exec", "--", "rustc", "-vV"]:
            pytest.fail("published executables must not depend on a compiler identity")
        return output(command, **kwargs)

    monkeypatch.setattr(bootstrap.urllib.request, "urlretrieve", checksums)
    monkeypatch.setattr(bootstrap.subprocess, "check_output", version)
    with pytest.raises(ValueError, match="published Harmos"):
        bootstrap.prepare_harmos(tmp_path, allow_install=True)
    assert not (tmp_path / ".cache/harmos-tool/receipt.json").exists()


def test_branch_artifact(tmp_path: Path, processes) -> None:
    calls, _source = processes
    _manifest(tmp_path, f'{{ git = "{bootstrap.HARMOS_REPOSITORY}", branch = "release/next" }}')
    source = f"git+{bootstrap.HARMOS_REPOSITORY}?branch=release%2Fnext#{REVISION}"
    _lock(tmp_path, source=source)
    binary = bootstrap.prepare_harmos(tmp_path, allow_install=True)
    assert calls[-1][6:10] == ["--git", bootstrap.HARMOS_REPOSITORY, "--rev", REVISION]
    assert bootstrap.prepare_harmos(tmp_path, allow_install=False) == binary
    assert len(calls) == 1


def test_artifact_refusal(tmp_path: Path, monkeypatch: pytest.MonkeyPatch, processes) -> None:
    calls, _source = processes
    _manifest(tmp_path, '"=0.1.0"')
    _lock(tmp_path)
    with pytest.raises(ValueError, match="consumers never rebuild"):
        bootstrap.prepare_harmos(tmp_path, allow_install=False)
    assert calls == []
    binary = bootstrap.prepare_harmos(tmp_path, allow_install=True)
    receipt = binary.parent.parent / "receipt.json"
    original = receipt.read_bytes()
    target = tmp_path / "receipt-target"
    target.write_bytes(original)
    receipt.unlink()
    receipt.symlink_to(target)
    with pytest.raises(ValueError, match="consumers never rebuild"):
        bootstrap.prepare_harmos(tmp_path, allow_install=False)
    receipt.unlink()
    receipt.write_bytes(original)
    target = tmp_path / "binary-target"
    binary.replace(target)
    binary.symlink_to(target)
    with pytest.raises(ValueError, match="consumers never rebuild"):
        bootstrap.prepare_harmos(tmp_path, allow_install=False)
    assert len(calls) == 1

    def omit(command, **_kwargs):
        assert command[:2] == ["mise", "install-into"]

    monkeypatch.setattr(bootstrap.subprocess, "run", omit)
    with pytest.raises(ValueError, match="omitted its required CLI"):
        bootstrap.prepare_harmos(tmp_path, allow_install=True)
    assert binary.is_symlink()
    assert receipt.read_bytes() == original


@pytest.mark.parametrize("namespace", [bootstrap.HARMOS_REPOSITORY, "crates-io"])
def test_local_override(tmp_path: Path, namespace: str, monkeypatch: pytest.MonkeyPatch, processes) -> None:
    calls, source = processes
    config, checkout = _override(tmp_path, namespace)
    assert not (tmp_path / "Cargo.lock").exists()
    binary = bootstrap.prepare_harmos(tmp_path, allow_install=True, cargo_config=config)
    assert calls[-1][6:8] == ["--path", str(checkout / "crates/cargo-harmos")]
    assert bootstrap.prepare_harmos(tmp_path, allow_install=False, cargo_config=config) == binary
    assert len(calls) == 1
    receipt = json.loads((binary.parent.parent / "receipt.json").read_text())
    assert receipt["identity"]["source"] == source[0]
    assert receipt["identity"]["cargo_config_sha256"] == hashlib.sha256(config.read_bytes()).hexdigest()

    original = config.read_bytes()
    config.write_bytes(original + b"\n# Changed Configuration\n")
    with pytest.raises(ValueError, match="consumers never rebuild"):
        bootstrap.prepare_harmos(tmp_path, allow_install=False, cargo_config=config)
    config.write_bytes(original)
    source[0] = "changed-source"
    with pytest.raises(ValueError, match="consumers never rebuild"):
        bootstrap.prepare_harmos(tmp_path, allow_install=False, cargo_config=config)
    install = bootstrap._run

    def mutate(command):
        install(command)
        source[0] = "changed-during-install"

    monkeypatch.setattr(bootstrap, "_run", mutate)
    with pytest.raises(ValueError, match="source changed"):
        bootstrap.prepare_harmos(tmp_path, allow_install=True, cargo_config=config)


@pytest.mark.parametrize("fault", ["CI", "GITHUB_ACTIONS", "DOKIME_CI_ROLE", "relative", "package", "tool", "ambiguous"])
def test_override_refusal(tmp_path: Path, fault: str, monkeypatch: pytest.MonkeyPatch, processes) -> None:
    config, checkout = _override(tmp_path, "crates-io")
    if fault in {"CI", "GITHUB_ACTIONS", "DOKIME_CI_ROLE"}:
        monkeypatch.setenv(fault, "true")
    elif fault == "relative":
        config.write_text('[patch.crates-io]\nharmos = { path = "relative" }\n')
    elif fault == "package":
        (checkout / "crates/harmos/Cargo.toml").write_text('[package]\nname = "wrong"\n')
    elif fault == "tool":
        (checkout / "crates/cargo-harmos/Cargo.toml").write_text('[package]\nname = "wrong"\n')
    else:
        config.write_text(config.read_text() + f'\n[patch."{bootstrap.HARMOS_REPOSITORY}"]\nharmos = {{ path = "unused" }}\n')
    with pytest.raises(ValueError):
        bootstrap.prepare_harmos(tmp_path, allow_install=True, cargo_config=config)
    assert processes[0] == []


@pytest.mark.parametrize("namespace", [bootstrap.HARMOS_REPOSITORY, "crates-io"])
def test_patch_source_refusal(tmp_path: Path, namespace: str, processes) -> None:
    config, _checkout = _override(tmp_path, namespace)
    declaration = (
        '"=0.1.0"' if namespace == bootstrap.HARMOS_REPOSITORY else f'{{ git = "{bootstrap.HARMOS_REPOSITORY}", rev = "{REVISION}" }}'
    )
    _manifest(tmp_path, declaration)
    with pytest.raises(ValueError, match="match the declared public dependency source"):
        bootstrap.prepare_harmos(tmp_path, allow_install=True, cargo_config=config)
    assert processes[0] == []


def test_binding_consumer(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    config = tmp_path / "override.toml"

    def consume(root, *, allow_install, cargo_config):
        assert root == tmp_path
        assert not allow_install
        assert cargo_config == config
        raise ValueError("missing prepared bytes")

    monkeypatch.setattr(bindings, "prepare_harmos", consume)
    with pytest.raises(ValueError, match="missing prepared bytes"):
        bindings.generate(None, root=tmp_path, cargo_config=config)


# === Private Functions === #


def _manifest(root: Path, declaration: str) -> None:
    text = "[workspace.dependencies]\n"
    text += "".join(f"{name} = {declaration}\n" for name in bootstrap.HARMOS_PACKAGES)
    (root / "Cargo.toml").write_text(text)


def _lock(root: Path, *, source: str = bootstrap.CRATES_SOURCE, version: str = "0.1.0") -> None:
    text = "version = 4\n"
    text += "".join(f'\n[[package]]\nname = "{name}"\nversion = "{version}"\nsource = "{source}"\n' for name in bootstrap.HARMOS_PACKAGES)
    (root / "Cargo.lock").write_text(text)


def _override(root: Path, namespace: str) -> tuple[Path, Path]:
    declaration = (
        f'{{ git = "{bootstrap.HARMOS_REPOSITORY}", rev = "{REVISION}" }}' if namespace == bootstrap.HARMOS_REPOSITORY else '"=0.1.0"'
    )
    _manifest(root, declaration)
    checkout = root / "external"
    for package in ("harmos", "cargo-harmos"):
        path = checkout / "crates" / package / "Cargo.toml"
        path.parent.mkdir(parents=True)
        path.write_text(f'[package]\nname = "{package}"\n')
    config = root / "override.toml"
    crate = (checkout / "crates/harmos").as_posix()
    config.write_text(f'[patch."{namespace}"]\nharmos = {{ path = "{crate}" }}\n')
    return config, checkout


# === Entry Point === #
