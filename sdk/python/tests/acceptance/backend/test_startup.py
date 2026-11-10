# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Start, Pair And Refuse Backends Through The Installed Native Wheel"""

# === Imports === #

# Standard Library Imports
import asyncio
import hashlib
import json
import socket
from collections.abc import AsyncIterator, Callable
from pathlib import Path

# Third Party Imports
import pytest

# Local Imports
from dokime import (
    Backend,
    BackendAddressUnavailable,
    BackendClosed,
    BackendRunning,
    DokimeClient,
    PluginIdConflict,
    _native,
)
from dokime.client.contracts import operation
from google.protobuf.message import Message

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def test_backend_pairing_and_shutdown(tmp_path: Path) -> None:
    async def story() -> None:
        pairing = tmp_path / "lab.pair"
        application = tmp_path / "application"
        async with Backend(port=0, application_root=application) as backend:
            assert backend.root == application / "data"
            endpoint = backend.endpoint
            async with backend.client() as client:
                identity = await client.connect()
            backend.export_pairing(pairing, label="Bench 3")
            document = json.loads(pairing.read_bytes())
            assert document["backend_identifier"] == identity.id
            assert document["label"] == "Bench 3"
            assert document["access_key"] and document["certificate_pem"]
            assert all(endpoint.startswith("https://") for endpoint in document["endpoints"])
            async with DokimeClient.from_pairing(pairing) as attached:
                assert (await attached.connect()).id == identity.id
            native = backend._inner
            assert native is not None
            stopping = native.close()
            refused = tmp_path / "refused.pair"
            with pytest.raises(BackendClosed):
                native.export_pairing(refused, "Closed")
            assert not refused.exists()
            await stopping
        original = pairing.read_bytes()
        with pytest.raises(BackendClosed):
            backend.export_pairing(pairing, label="Closed")
        assert pairing.read_bytes() == original
        with socket.socket() as connection:
            assert connection.connect_ex(("127.0.0.1", int(endpoint.rsplit(":", 1)[1]))) != 0
        async with Backend(port=0, application_root=application) as restarted, restarted.client() as client:
            assert (await client.connect()).id == identity.id

    asyncio.run(story())


def test_backend_root_owned(tmp_path: Path) -> None:
    async def story() -> None:
        root = tmp_path / "evidence"
        async with Backend(port=0, root=root, application_root=tmp_path / "application") as backend:
            with pytest.raises(BackendRunning) as caught:
                async with Backend(port=0, root=root, application_root=tmp_path / "other"):
                    pytest.fail("owned root was admitted")
            assert caught.value.root == root
            assert str(root) in str(caught.value)
            async with backend.client() as client:
                await client.connect()

    asyncio.run(story())


def test_backend_port_refusal_releases_root(tmp_path: Path) -> None:
    async def story(port: int) -> None:
        root = tmp_path / "data"
        application = tmp_path / "application"
        with pytest.raises(BackendAddressUnavailable) as caught:
            async with Backend(port=port, root=root, application_root=application):
                pytest.fail("occupied port was admitted")
        assert caught.value.address == f"127.0.0.1:{port}"
        assert caught.value.address in str(caught.value)
        async with Backend(port=0, root=root, application_root=application) as backend, backend.client() as client:
            await client.connect()

    with socket.socket() as occupied:
        occupied.bind(("127.0.0.1", 0))
        occupied.listen()
        asyncio.run(story(occupied.getsockname()[1]))


def test_backend_exception_releases_root(tmp_path: Path) -> None:
    async def story() -> None:
        application = tmp_path / "application"
        with pytest.raises(ArithmeticError):
            async with Backend(port=0, application_root=application):
                raise ArithmeticError
        async with Backend(port=0, application_root=application) as backend, backend.client() as client:
            await client.connect()

    asyncio.run(story())


@pytest.mark.parametrize("enabled", [True, False])
def test_backend_installed_plugins_and_id_clash(tmp_path: Path, artifact: Callable[[str], Path], enabled: bool) -> None:
    async def story() -> None:
        application = tmp_path / "application"
        bundle = artifact("fixtures/plugins/plugin-obc.dokime").read_bytes()
        async with Backend(port=0, application_root=application) as backend, backend.client() as client:
            await _install(client, artifact("plugins/plugin-host.dokime").read_bytes())
            installed = await _install(client, bundle)
            plugin_id = installed.metadata.owners[0]
            install_id = installed.install_id
        python = _native.EmbeddedProvider("python_lab", "1.0.0", "Python Lab")
        async with Backend(port=0, plugins=[python], application_root=application) as backend, backend.client() as client:
            catalog = await client.call(operation("Plugins", "ListAdmittedPlugins"), {"page_size": 100})
            assert {plugin_id, "python_lab"} <= {plugin.plugin_id for plugin in catalog.plugins}
            suites = [suite async for suite in client.suites()]
            assert any(suite.identifier.startswith("obc.") for suite in suites)
            if not enabled:
                await client.call(operation("Plugins", "SetEnabled"), {"install_id": install_id, "enabled": False})
        clash = _native.EmbeddedProvider(plugin_id, "1.0.0", "Conflicting Python Plugin")
        with pytest.raises(PluginIdConflict) as caught:
            async with Backend(port=0, plugins=[clash], application_root=application):
                pytest.fail("installed ID was shadowed")
        assert caught.value.plugin_id == plugin_id
        assert plugin_id in str(caught.value)
        async with Backend(port=0, application_root=application) as backend, backend.client() as client:
            catalog = await client.call(operation("Plugins", "ListAdmittedPlugins"), {"page_size": 100})
            assert "python_lab" not in {plugin.plugin_id for plugin in catalog.plugins}
            installations = await client.call(operation("Plugins", "ListInstallations"), {"page_size": 100})
            retained = next(entry for entry in installations.installations if entry.install_id == install_id)
            assert retained.active == enabled
            assert not retained.boot_failed

    asyncio.run(story())


# === Private Functions === #


async def _install(client: DokimeClient, bundle: bytes) -> Message:
    install = operation("Plugins", "Install")
    input_type = install.input
    assert input_type is not None

    async def frames() -> AsyncIterator[Message]:
        frame = input_type()
        frame.frame.Header.field_0.size_bytes = len(bundle)
        frame.frame.Header.field_0.sha256 = hashlib.sha256(bundle).hexdigest()
        yield frame
        for offset in range(0, len(bundle), 65536):
            frame = input_type()
            frame.frame.Chunk.field_0.offset = offset
            frame.frame.Chunk.field_0.data = bundle[offset : offset + 65536]
            yield frame

    return await client.upload(install, install.request(), frames())


# === Entry Point === #
