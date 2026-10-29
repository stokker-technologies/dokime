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
from dokime_tools.package import plugins

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


@pytest.mark.parametrize("shipped", [[], ["instruments-plugin", "obc-plugin"]])
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

    expected = sorted({"obc-plugin", *shipped})
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

# === Entry Point === #
