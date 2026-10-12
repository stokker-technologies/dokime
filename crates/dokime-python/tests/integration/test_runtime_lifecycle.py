# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Check backend ownership and cancellation across the async Python boundary."""

# === Imports === #

# Standard Library Imports
from collections.abc import Callable
import asyncio
import socket
from contextlib import asynccontextmanager
from pathlib import Path
from urllib.parse import urlsplit

# Third Party Imports
import pytest

# Local Imports
import dokime
from dokime import Runtime, _native
from dokime.runtime import BearerGrant, Role, ServerConfig, TlsIdentity
from dokime.client import DokimeClient

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def case_runtime_connection(tmp_path: Path) -> None:
    async def run() -> None:
        port = _port()
        endpoint = f"http://127.0.0.1:{port}"
        runtime = Runtime(application_root=tmp_path)
        attached = DokimeClient(endpoint)
        try:
            client = await runtime.start(server_config=ServerConfig.local(port=port))
            first = await client.connect()
            assert await attached.connect() == first
            await attached.close()
            assert [suite async for suite in client.suites()] == []
            with pytest.raises(RuntimeError, match="already started"):
                await runtime.start()
            assert await client.connect() == first
            await asyncio.gather(runtime.close(), runtime.close())
            with pytest.raises(OSError):
                await asyncio.to_thread(_connect, endpoint)
            with pytest.raises(RuntimeError, match="closed"):
                await runtime.start()
        finally:
            await attached.close()
            await runtime.close()

        reopened = Runtime(application_root=tmp_path)
        try:
            client = await reopened.start(server_config=ServerConfig.local(port=port))
            assert await client.connect() != first
            assert [suite async for suite in client.suites()] == []
        finally:
            await reopened.close()

    asyncio.run(asyncio.wait_for(run(), timeout=20))


@pytest.mark.parametrize("phase", ["start", "close", "native_start"])
def test_cancelled_owned(tmp_path: Path, phase: str) -> None:
    async def run() -> None:
        opening = asyncio.Event()
        release_start = asyncio.Event()
        closing = asyncio.Event()
        release_close = asyncio.Event()
        finished = []

        class Probe(dokime.TestCaseTemplate):
            template_id = "lifecycle.probe"

            @dokime.step
            async def measure(self, ctx):
                pass

        class Suite(dokime.TestSuite):
            suite_id = "lifecycle.suite"
            members = (Probe,)

            @asynccontextmanager
            async def resources(self):
                opening.set()
                await release_start.wait()
                try:
                    yield None
                finally:
                    closing.set()
                    await release_close.wait()
                    finished.append("closed")

        runtime = Runtime(application_root=tmp_path)
        if phase == "native_start":
            from dokime._provider import Provider

            native = _native.Runtime(application_root=tmp_path)
            await native.register(Provider((Suite,)))
            start = native.start
            close = native.close
        else:
            await runtime.register(Suite)
            start = runtime.start
            close = runtime.close

        try:
            if phase == "close":
                release_start.set()
                await start()
                waiter = asyncio.ensure_future(close())
                await closing.wait()
            else:
                waiter = asyncio.ensure_future(start())
                await opening.wait()
            waiter.cancel("original cancellation")
            release_start.set()
            if phase == "native_start":
                with pytest.raises(asyncio.CancelledError):
                    await waiter
                waiter = asyncio.ensure_future(close())
            await closing.wait()
            waiter.cancel("another cancellation")
            assert not finished
            # Every caller observes the retained close operation. Cancelling a
            # waiter must not cancel the provider's awaited finalizer.
            joining = asyncio.ensure_future(close())
            release_close.set()
            await joining
            with pytest.raises(asyncio.CancelledError) as cancelled:
                await waiter
            if phase == "start":
                assert cancelled.value.args == ("original cancellation",)
            assert finished == ["closed"]
            await close()
        finally:
            release_start.set()
            release_close.set()
            await close()
            await runtime.close()

    asyncio.run(asyncio.wait_for(run(), timeout=20))


def case_server_secrets(
    tmp_path: Path,
) -> None:
    token = "python-native-grant-012345678901234567890123456789"
    grant = BearerGrant("python.fixture", Role.Control, token)
    assert token not in repr(grant)
    identity = TlsIdentity.from_pem(b"certificate", b"private-key")
    assert "private-key" not in repr(identity)
    with pytest.raises(ValueError, match="bearer tokens"):
        BearerGrant("python.fixture", Role.Read, "short")
    with pytest.raises(ValueError, match="at least one grant"):
        ServerConfig.remote(identity, [])

    async def run() -> None:
        for name, configuration, message in [
            ("host", ServerConfig.local(host="0.0.0.0"), "non-loopback"),
            ("limit", ServerConfig.local(max_message_bytes=0), "message limit"),
        ]:
            invalid = Runtime(application_root=tmp_path / name)
            try:
                with pytest.raises(ValueError, match=message):
                    await invalid.start(server_config=configuration)
            finally:
                await invalid.close()
        runtime = Runtime(application_root=tmp_path / "first")
        other = Runtime(application_root=tmp_path / "other")
        try:
            configuration = ServerConfig.local(grants=[grant])
            client = await runtime.start(server_config=configuration, token=token)
            assert [suite async for suite in client.suites()] == []
            with pytest.raises(ValueError, match="already consumed"):
                await other.start(server_config=configuration, token=token)
            assert [suite async for suite in client.suites()] == []
        finally:
            await other.close()
            await runtime.close()

    asyncio.run(asyncio.wait_for(run(), timeout=20))


@pytest.mark.parametrize(
    "mode", ["early", "rejected_frames", "eof", "caller", "client", "producer"]
)
def test_upload_finalizers(tmp_path: Path, mode: str) -> None:
    from dokime._native.client import Client, ClientError, Target
    from dokime._uploads import upload

    async def run() -> None:
        port = _port()
        runtime = Runtime(application_root=tmp_path)
        native = None
        task = None
        started = asyncio.Event()
        finishing = asyncio.Event()
        release = asyncio.Event()
        finished = asyncio.Event()

        async def frames():
            try:
                started.set()
                if mode == "producer":
                    raise ValueError("producer failed")
                if mode == "eof":
                    return
                if mode == "rejected_frames":
                    for _ in range(1000):
                        yield b""
                if mode == "early":
                    yield b""
                await asyncio.Event().wait()
                yield b""
            finally:
                finishing.set()
                await release.wait()
                finished.set()

        try:
            await runtime.start(server_config=ServerConfig.local(port=port))
            native = await Client.connect(Target(f"http://127.0.0.1:{port}"))
            task = asyncio.create_task(
                upload(native, "/dokime.v1.Plugins/Stage", frames())
            )
            await started.wait()
            if mode == "caller":
                task.cancel()
            elif mode == "client":
                await native.close()
            await finishing.wait()
            if mode == "caller":
                task.cancel()
                await asyncio.sleep(0)
                task.cancel()
                await asyncio.sleep(0)
            assert not task.done()
            release.set()

            if mode == "caller":
                with pytest.raises(asyncio.CancelledError):
                    await task
            elif mode == "producer":
                with pytest.raises(ValueError, match="producer failed"):
                    await task
            else:
                with pytest.raises(ClientError) as failure:
                    await task
                assert failure.value.kind == ("closed" if mode == "client" else "rpc")
                if mode != "client":
                    assert failure.value.code == 3
            assert finished.is_set()
        finally:
            release.set()
            if task is not None:
                task.cancel()
                await asyncio.gather(task, return_exceptions=True)
            if native is not None:
                await native.close()
            await runtime.close()

    asyncio.run(asyncio.wait_for(run(), timeout=20))


def case_upload_bytes(tmp_path: Path) -> None:
    from dokime._native.client import Client, Target
    from dokime._uploads import upload
    from dokime.client._generated.dokime.v1 import content_pb2, execution_pb2

    async def run() -> None:
        ready = asyncio.Event()
        release = asyncio.Event()
        produced = asyncio.Event()

        class Hold(dokime.TestCaseTemplate):
            template_id = "upload.hold"

            @dokime.step
            async def hold(self, ctx):
                ready.set()
                await release.wait()

        class Lab(dokime.TestSuite):
            suite_id = "upload.lab"
            members = (Hold,)

        port = _port()
        runtime = Runtime(application_root=tmp_path)
        native = None
        try:
            await runtime.register(Lab)
            client = await runtime.start(server_config=ServerConfig.local(port=port))
            native = await Client.connect(Target(f"http://127.0.0.1:{port}"))
            run_cycle = await client.new_cycle(
                suite="upload.lab", template="upload.hold"
            )
            run = await run_cycle.run(request_key="upload")
            await ready.wait()
            members = await client._transport.unary(
                "Execution",
                "ListExecutions",
                execution_pb2.ListExecutionsRequest(cycle_id=run.cycle_id, limit=10),
            )
            scope = content_pb2.ArtifactScope(
                cycle_id=run.cycle_id, execution_id=members.executions[0].execution_id
            )
            data = b"charger measurement\n" * 100

            async def frames():
                try:
                    yield content_pb2.UploadArtifactFrame(
                        header=content_pb2.UploadArtifactHeader(
                            scope=scope,
                            filename="measurement.txt",
                            size_bytes=len(data),
                        )
                    ).SerializeToString()
                    yield content_pb2.UploadArtifactFrame(
                        chunk=content_pb2.ArtifactChunk(offset=0, data=data)
                    ).SerializeToString()
                finally:
                    await asyncio.sleep(0)
                    produced.set()

            artifact = content_pb2.ArtifactMetadata.FromString(
                await upload(native, "/dokime.v1.Content/UploadArtifact", frames())
            )
            assert produced.is_set()
            assert artifact.size_bytes == len(data)
            stream = await native.stream(
                "/dokime.v1.Content/DownloadArtifact",
                content_pb2.DownloadArtifactRequest(
                    artifact_id=artifact.artifact_id
                ).SerializeToString(),
            )
            received = bytearray()
            try:
                while (payload := await stream.message()) is not None:
                    frame = content_pb2.ArtifactFrame.FromString(payload)
                    if frame.WhichOneof("payload") == "chunk":
                        received.extend(frame.chunk.data)
            finally:
                await stream.close()
            assert received == data
            release.set()
            await run.wait(timeout=15)
        finally:
            release.set()
            if native is not None:
                await native.close()
            await runtime.close()

    asyncio.run(asyncio.wait_for(run(), timeout=25))


# === Private Functions === #


def _port() -> int:
    with socket.socket() as reservation:
        reservation.bind(("127.0.0.1", 0))
        return reservation.getsockname()[1]


def _connect(endpoint: str) -> None:
    address = urlsplit(endpoint)
    with socket.create_connection((address.hostname, address.port), timeout=5):
        pass


# === Entry Point === #


def case_native_errors(tmp_path: Path) -> None:
    from dokime._native.client import Client, ClientError, Target
    from dokime.client._generated.dokime.v1 import backend_pb2, catalog_pb2

    async def run() -> None:
        port = _port()
        runtime = Runtime(application_root=tmp_path)
        native = None
        try:
            existing = await runtime.start(server_config=ServerConfig.local(port=port))
            identity = await existing.connect()
            native = await Client.connect(Target(f"http://127.0.0.1:{port}"))
            snapshot = native.backend
            assert snapshot.backend_id == identity.id
            assert snapshot.epoch == identity.epoch
            wire_identity = backend_pb2.BackendIdentity.FromString(
                await native.unary("/dokime.v1.Backend/GetIdentity", b"")
            )
            capabilities = backend_pb2.BackendCapabilities.FromString(
                await native.unary("/dokime.v1.Backend/GetCapabilities", b"")
            )
            assert snapshot.display_name == wire_identity.display_name
            assert snapshot.product_version == wire_identity.product_version
            assert snapshot.api_version == wire_identity.api.version
            assert (
                snapshot.descriptor_sha256 == wire_identity.api.descriptor_sha256.hex()
            )
            assert isinstance(snapshot.methods, tuple)
            assert frozenset(snapshot.methods) == frozenset(capabilities.methods)
            assert (
                snapshot.limits.max_request_bytes
                == capabilities.limits.max_request_bytes
            )
            assert (
                snapshot.limits.max_response_bytes
                == capabilities.limits.max_response_bytes
            )
            for field in (
                "max_content_bytes",
                "max_chunk_bytes",
                "max_page_items",
                "max_stream_buffer_items",
            ):
                expected = (
                    getattr(capabilities.limits, field)
                    if capabilities.limits.HasField(field)
                    else None
                )
                assert getattr(snapshot.limits, field) == expected
            with pytest.raises(AttributeError):
                setattr(snapshot, "backend_id", "changed")
            with pytest.raises(AttributeError):
                setattr(snapshot.limits, "max_request_bytes", 0)
            await native.verify()
            health = backend_pb2.BackendHealth.FromString(
                await native.unary("/dokime.v1.Backend/GetHealth", b"")
            )
            assert health.backend_id == identity.id

            missing = catalog_pb2.GetSuiteRequest(
                suite_id="00000000-0000-0000-0000-000000000000"
            )
            with pytest.raises(ClientError) as rpc:
                await native.unary(
                    "/dokime.v1.Catalog/GetSuite", missing.SerializeToString()
                )
            assert rpc.value.kind == "rpc"
            assert rpc.value.code == 5
            assert isinstance(rpc.value.details, bytes)

            with pytest.raises(ValueError):
                await native.unary("/dokime.v1.Backend/GetHealth", b"\x80")
            with pytest.raises(ValueError, match="unknown unary"):
                await native.unary("/unknown/Method", b"")

            logs = await native.stream("/dokime.v1.Backend/WatchLogs", b"")
            waiting = asyncio.ensure_future(logs.message())
            await asyncio.sleep(0)
            await logs.close()
            try:
                await waiting
            except ClientError as error:
                assert error.kind == "closed"
            with pytest.raises(ClientError) as closed_stream:
                await logs.message()
            assert closed_stream.value.kind == "closed"
            await logs.close()
            assert await native.unary("/dokime.v1.Backend/GetHealth", b"")
            with pytest.raises(ValueError, match="unknown streaming"):
                await native.stream("/unknown/Method", b"")
            with pytest.raises(ValueError):
                await native.stream("/dokime.v1.Backend/WatchLogs", b"\x80")

            with pytest.raises(ValueError, match="unknown upload"):
                await native.upload("/unknown/Method")
            upload = await native.upload("/dokime.v1.Plugins/Stage")
            try:
                with pytest.raises(ValueError):
                    await upload.send(b"\x80")
                with pytest.raises(ValueError, match="message limit"):
                    upload.send(b"x" * (16 * 1024 * 1024 + 1))
                await upload.send(b"")
                with pytest.raises(ClientError) as rejected_upload:
                    await upload.finish()
                assert rejected_upload.value.kind == "rpc"
                assert rejected_upload.value.code == 3
            finally:
                await upload.close()
            assert await native.unary("/dokime.v1.Backend/GetHealth", b"")

            upload = await native.upload("/dokime.v1.Plugins/Stage")
            logs = await native.stream("/dokime.v1.Backend/WatchLogs", b"")
            closing = asyncio.ensure_future(native.close())
            closing.cancel()
            with pytest.raises(ClientError) as refused:
                await native.unary("/dokime.v1.Backend/GetHealth", b"")
            assert refused.value.kind == "closed"
            assert refused.value.code is None
            assert refused.value.details == b""
            await native.close()
            with pytest.raises(ClientError) as closed_upload:
                await upload.send(b"")
            assert closed_upload.value.kind == "closed"
            await upload.close()
            with pytest.raises(ClientError) as closed_stream:
                await logs.message()
            assert closed_stream.value.kind == "closed"
            await logs.close()
            assert native.backend.backend_id == snapshot.backend_id
            assert native.backend.methods == snapshot.methods
            assert snapshot.limits.max_request_bytes > 0
            assert [suite async for suite in existing.suites()] == []
        finally:
            if native is not None:
                await native.close()
            await runtime.close()

    asyncio.run(asyncio.wait_for(run(), timeout=20))


@pytest.mark.parametrize("mode", ["native", "public"])
def test_failed_retry(tmp_path: Path, mode: str) -> None:
    from dokime._provider import Provider

    async def run() -> None:
        class RefusingProvider(Provider):
            def __init__(self) -> None:
                super().__init__(())
                self.refuse = True
                self.attempts = 0

            async def start(self) -> None:
                raise ValueError("initialization refused")

            async def close(self) -> None:
                self.attempts += 1
                if self.refuse:
                    raise ValueError("cleanup refused")

        provider = RefusingProvider()
        if mode == "native":
            native = _native.Runtime(application_root=tmp_path)
            runtime = native
        else:
            public = Runtime(application_root=tmp_path)
            native = getattr(public, "_native")
            runtime = public
        await native.register(provider)
        try:
            with pytest.raises(ValueError, match="initialization refused") as failed:
                await runtime.start()
            assert "cleanup refused" in str(failed.value)
            if mode == "public":
                assert failed.value.__cause__ is not None
                assert "cleanup refused" in str(failed.value.__cause__)

            with pytest.raises(ValueError, match="cleanup refused"):
                await runtime.close()
            attempts = provider.attempts
            assert attempts > 0
            provider.refuse = False
            await runtime.close()
            assert provider.attempts == attempts + 1
            await runtime.close()
            assert provider.attempts == attempts + 1

            reopened = Runtime(application_root=tmp_path)
            try:
                await reopened.start()
            finally:
                await reopened.close()
        finally:
            provider.refuse = False
            await runtime.close()

    asyncio.run(asyncio.wait_for(run(), timeout=20))


@pytest.mark.parametrize(
    "scenario",
    [
        pytest.param(case_runtime_connection, id="runtime_connection"),
        pytest.param(case_server_secrets, id="server_secrets"),
        pytest.param(case_upload_bytes, id="upload_bytes"),
        pytest.param(case_native_errors, id="native_errors"),
    ],
)
def test_contract(scenario: Callable[..., None], tmp_path: Path) -> None:
    scenario(tmp_path)
