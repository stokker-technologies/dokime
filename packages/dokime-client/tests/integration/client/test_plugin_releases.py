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
import hashlib
import json
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


def test_release_feed_uses_inspected_identity_targets_and_complete_digest(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    bundle = tmp_path / "instruments.dokime"
    bundle.write_bytes(b"complete inspected bundle")
    manifest = {"identity": {"name": "instruments", "version": "1.2.3"}, "files": [{"executable": {"target": "x86_64-pc-windows-msvc"}}]}
    calls = []

    def inspect(command: list[str], **kwargs: object) -> SimpleNamespace:
        calls.append(command)
        assert kwargs["check"] is True
        return SimpleNamespace(stdout=json.dumps({"manifest": manifest}))

    monkeypatch.setattr(plugins.subprocess, "run", inspect)
    tool = tmp_path / "cargo-dokime"
    assets = [(bundle, "https://example.test/instruments.dokime")]
    feed = plugins.release_feed(assets, tool=tool, requires_dokime="=0.1.0")

    assert calls == [[str(tool), "dokime", "sidecar", "inspect", str(bundle)]]
    assert (feed["format_version"], feed["package"], feed["version"]) == (1, "instruments", "1.2.3")
    assert feed["assets"] == [
        {"target": "x86_64-pc-windows-msvc", "url": assets[0][1], "sha256": hashlib.sha256(bundle.read_bytes()).hexdigest()}
    ]
    with pytest.raises(ValueError, match="distinct executable targets"):
        plugins.release_feed(assets * 2, tool=tool, requires_dokime="=0.1.0")
    with pytest.raises(ValueError, match="credential-free HTTPS"):
        plugins.release_feed([(bundle, "https://token@example.test/plugin")], tool=tool, requires_dokime="=0.1.0")


# === Private Functions === #


@pytest.mark.parametrize("fixture", [False, True])
def test_plugin_exports_select_shipping_or_fixture_packages(tmp_path: Path, monkeypatch: pytest.MonkeyPatch, fixture: bool) -> None:
    output = tmp_path / ("fixtures/plugins" if fixture else "plugins")
    native = plugins.paths.target()
    policy = f'[[release.plugins]]\npackage = "dokime-host"\ntargets = {{ {native} = "native-abi" }}\n'
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
