# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Generate Release Metadata From Canonically Inspected Bundles"""

# === Imports === #

# Standard Library Imports
import json
import subprocess
import tarfile
import zipfile
from contextlib import nullcontext
from pathlib import Path
from types import SimpleNamespace

# Third Party Imports
import pytest

# Local Imports
from dokime_tools.build.rust import tests
from dokime_tools.package import plugins

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


@pytest.mark.parametrize("shipped", [[], ["instruments-plugin", "instruments-plugin"]])
def test_collect_declared_plugins_once(tmp_path: Path, monkeypatch: pytest.MonkeyPatch, shipped: list[str]) -> None:
    native = plugins.paths.target()
    declarations = [f'[[release.plugins]]\npackage = "{package}"\ntargets = {{ {native} = "native-abi" }}\n' for package in shipped]
    declarations.append('[[release.plugins]]\npackage = "foreign-plugin"\ntargets = { foreign = "foreign-abi" }\n')
    (tmp_path / "stkr.toml").write_text("\n".join(declarations), encoding="utf-8")
    tool = tmp_path / "cargo-dokime"
    messages = tmp_path / "compiler.json"
    calls = []

    def collect(command: list[str], **kwargs: object) -> SimpleNamespace:
        calls.append(command)
        assert kwargs["check"] is True
        if "--build-json" in command:
            build_json = Path(command[command.index("--build-json") + 1])
            package = build_json.name.removesuffix(".build.json")
            return SimpleNamespace(stdout=json.dumps([{"path": f"plugins/{package}.dokime"}]))
        return SimpleNamespace(stdout="")

    monkeypatch.setattr(plugins.subprocess, "run", collect)

    bundles = plugins.prepare(tmp_path, messages, "release", tool=tool, output=tmp_path / "plugins")

    expected = sorted(set(shipped))
    assert bundles == [tmp_path / "plugins" / f"{package}.dokime" for package in expected]
    assert calls[0] == ["cargo", "metadata", "--locked", "--format-version", "1"]
    collectors = [command for command in calls[1:] if "--package" in command]
    assert [command[command.index("--package") + 1] for command in collectors] == expected
    assert all(command[:4] == [str(tool), "dokime", "sidecar", "build"] for command in collectors)
    assert all(command[command.index("--from-cargo-json") + 1] == str(messages) for command in collectors)
    assert len(calls) == 1 + 2 * len(expected)


@pytest.mark.parametrize("returncode", [0, 23])
def test_child_diagnostics(tmp_path: Path, monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str], returncode: int) -> None:
    (tmp_path / "stkr.toml").write_text("", encoding="utf-8")
    tool = tmp_path / "cargo-dokime"
    bundle = tmp_path / "plugin.dokime"
    bundle.write_bytes(b"bundle")
    child_stderr = "llvm-objcopy is missing; install the llvm-tools-preview component"
    child_stdout = "child diagnostic on stdout"
    commands = []

    def packager(command: list[str], **kwargs: object) -> SimpleNamespace:
        commands.append(command)
        if "package" not in command:
            return SimpleNamespace(stdout="")
        if returncode:
            raise subprocess.CalledProcessError(returncode, command, output=child_stdout, stderr=child_stderr)
        return SimpleNamespace(stdout=json.dumps([]), stderr=child_stderr)

    monkeypatch.setattr(plugins.subprocess, "run", packager)

    def invoke() -> None:
        plugins.prepare(tmp_path, tmp_path / "compiler.json", "dev", tool=tool, output=tmp_path / "plugins", packages=plugins.FIXTURES)

    expected = pytest.raises(subprocess.CalledProcessError) if returncode else nullcontext()
    with expected as raised:
        invoke()

    captured = capsys.readouterr()
    if raised is None:
        assert captured.out == captured.err == ""
        return
    assert raised.value.returncode == returncode
    assert raised.value.cmd == commands[-1]
    assert child_stderr in captured.err
    assert child_stdout in captured.err
    assert subprocess.list2cmdline(commands[-1]) in captured.err
    assert str(returncode) in captured.err


# === Private Functions === #


@pytest.mark.parametrize("target", ["x86_64-pc-windows-msvc", "x86_64-unknown-linux-gnu"])
def test_sidecars_archive(tmp_path: Path, target: str) -> None:
    (tmp_path / "stkr.toml").write_text(
        '[[release.plugins]]\npackage="crate-name"\nkind="sidecar"\npath="rust/plugins/opaque.dokime"\n'
        f'[release.plugins.targets]\nnative="{target}"\n',
        encoding="utf-8",
    )
    build = tmp_path / "build"
    (build / "plugins").mkdir(parents=True)
    (build / "plugins/opaque.dokime").write_bytes(b"verified shipping bundle")
    (build / "plugins/fixture.dokime").write_bytes(b"excluded fixture")
    archive = plugins.archive(tmp_path, build, tmp_path / "output", "1.2.3", target)
    assert archive is not None
    suffix = ".zip" if "windows" in target else ".tar.gz"
    assert archive.name == f"dokime-sidecars-1.2.3-{target}{suffix}"
    if suffix == ".zip":
        with zipfile.ZipFile(archive) as opened:
            assert opened.namelist() == ["opaque.dokime"]
            assert opened.read("opaque.dokime") == b"verified shipping bundle"
    else:
        with tarfile.open(archive) as opened:
            assert opened.getnames() == ["opaque.dokime"]
            member = opened.extractfile("opaque.dokime")
            assert member is not None
            assert member.read() == b"verified shipping bundle"


@pytest.mark.parametrize("fixture", [False, True])
def test_plugin_exports_select_shipping_or_fixture_packages(tmp_path: Path, monkeypatch: pytest.MonkeyPatch, fixture: bool) -> None:
    output = tmp_path / ("fixtures/plugins" if fixture else "plugins")
    native = plugins.paths.target()
    policy = f'[[release.plugins]]\npackage = "dokime-host"\npath = "rust/plugins/host.dokime"\ntargets = {{ {native} = "native-abi" }}\n'
    (tmp_path / "stkr.toml").write_text(policy, encoding="utf-8")
    selected = []

    def run(command: list[str], **kwargs: object) -> SimpleNamespace:
        if command[:2] == ["cargo", "metadata"]:
            kwargs["stdout"].write("{}")
        elif "build" in command:
            selected.append(command[command.index("--package") + 1])
            kwargs["stdout"].write("{}")
        else:
            path = output / f"{selected[-1]}.dokime"
            path.write_bytes(b"packaged executable")
            item = {"path": str(path)}
            return SimpleNamespace(stdout=json.dumps([item]))
        return SimpleNamespace()

    monkeypatch.setattr(plugins.subprocess, "run", run)
    arguments = {"packages": plugins.FIXTURES} if fixture else {}
    staged = plugins.prepare(tmp_path, tmp_path / "compiler.json", "dev", tool=tmp_path / "cargo-dokime", output=output, **arguments)
    assert selected == (["obc-plugin"] if fixture else ["dokime-host"])
    assert [path.name for path in staged] == (["obc-plugin.dokime"] if fixture else ["host.dokime"])
    assert staged[0].read_bytes() == b"packaged executable"


def test_test_archive_packages_fixtures_with_its_own_compiled_collector(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    built = tmp_path / "tests/archive.tar.zst"
    built.parent.mkdir()
    collector = tmp_path / "target/debug/cargo-dokime"
    artifact = {"reason": "compiler-artifact", "target": {"name": "cargo-dokime", "kind": ["bin"]}, "profile": {"test": False}}
    record = json.dumps(artifact | {"executable": str(collector)})
    built.with_name("compiler.json").write_text(record, encoding="utf-8")
    prepared = []

    def run(command: list[str], **kwargs: object) -> SimpleNamespace:
        return SimpleNamespace()

    def prepare(root: Path, messages: Path, profile: str, *, tool: Path, output: Path, packages: tuple[str, ...]) -> list[Path]:
        prepared.append((messages, profile, tool, packages))
        return []

    monkeypatch.setattr(tests, "ROOT", tmp_path)
    monkeypatch.setattr(tests.configuration, "ROOT", tmp_path)
    monkeypatch.setattr(tests.configuration, "features", lambda profile, **kwargs: [])
    monkeypatch.setattr(tests.cargo, "archive", lambda root, profile, extra: built)
    monkeypatch.setattr(tests.subprocess, "run", run)
    monkeypatch.setattr(tests.plugins, "prepare", prepare)
    monkeypatch.setattr(tests.archive, "stamp", lambda root, profile, names: None)
    tests.build("dev")
    assert prepared == [(built.with_name("compiler.json"), "dev", collector, plugins.FIXTURES)]


# === Entry Point === #
