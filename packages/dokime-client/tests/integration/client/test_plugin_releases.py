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


# === Entry Point === #
