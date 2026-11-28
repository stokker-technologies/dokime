# SPDX-License-Identifier: Apache-2.0

"""Settings Classes Describe Library Values and Read One Activation Snapshot."""

# === Imports === #

# Standard Library Imports
from datetime import timedelta
from unittest import IsolatedAsyncioTestCase
from unittest.mock import AsyncMock

# Third Party Imports
import harmos
from harmos import plugin
from harmos._generated.proto.host.v1.plugins.settings_pb2 import SettingsListResponse
from harmos._generated.proto.plugin.v1.invocation_pb2 import Request

# Local Imports
from dokime_sdk import Plugin
from dokime_sdk.operations import Context, Resource, Setting, Settings

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class ConnectionSettings(Settings):
    api_address: str = Setting.text("Remote API Address", default="127.0.0.1:30059")
    dll_timeout: timedelta = Setting.duration("DLL Host Timeout", default=timedelta(seconds=5), min=timedelta(), max=timedelta(seconds=30))
    enabled: bool = Setting.flag("Enabled", default=False)
    count: int = Setting.integer("Count", default=0, min=0, max=10)
    threshold: float = Setting.number("Threshold", default=0.0, min=0.0, max=1.0)


class SettingsTests(IsolatedAsyncioTestCase):
    def test_declarations_use_the_library_contract_including_zero_bounds(self) -> None:
        fields = ConnectionSettings._fields()
        for name, field in fields.items():
            declaration = field._definition(name)
            self.assertIsInstance(declaration, harmos.SettingDeclaration)
            self.assertEqual(declaration.name, name)
            self.assertEqual(declaration.default, field.default)
            self.assertEqual(declaration.HasField("minimum"), name in ("dll_timeout", "count", "threshold"))
            self.assertEqual(declaration.HasField("maximum"), name in ("dll_timeout", "count", "threshold"))
        count = fields["count"].minimum
        threshold = fields["threshold"].minimum
        timeout = fields["dll_timeout"].maximum
        assert count is not None and threshold is not None and timeout is not None
        self.assertEqual(count.integer, 0)
        self.assertEqual(threshold.number, 0.0)
        self.assertEqual(timeout.duration.ToTimedelta(), timedelta(seconds=30))

    async def test_activation_snapshots_are_typed_synchronous_and_refreshed_before_resources_open(self) -> None:
        observed: list[str] = []
        released: list[str] = []

        class RemoteApi(Resource):
            async def open(self, ctx: Context) -> None:
                self.ctx = ctx
                observed.append(ctx.settings(ConnectionSettings).api_address)

            async def close(self) -> None:
                released.append(self.ctx.settings(ConnectionSettings).api_address)

        class ConnectionPlugin(Plugin):
            version = "1.0.0"
            settings = (ConnectionSettings,)
            resources = (RemoteApi,)

            async def initialize(self, ctx: Context) -> None:
                await ctx.resource(RemoteApi)

        owner = ConnectionPlugin()
        transport = plugin.Context(Request(attempt=1), owner.identifier, {})
        values = harmos.SettingValues(values={name: field.default for name, field in ConnectionSettings._fields().items()})
        transport.settings = AsyncMock(return_value=SettingsListResponse(value=values))
        context = Context(transport, owner)
        with self.assertRaisesRegex(ValueError, "Activate the Plugin"):
            context.settings(ConnectionSettings)
        await owner._lifecycle("initialize")(object(), transport)
        snapshot = context.settings(ConnectionSettings)
        self.assertIsInstance(snapshot, ConnectionSettings)
        self.assertEqual(snapshot.api_address, "127.0.0.1:30059")
        self.assertEqual(snapshot.dll_timeout, timedelta(seconds=5))
        self.assertFalse(snapshot.enabled)
        self.assertEqual(snapshot.count, 0)
        self.assertEqual(snapshot.threshold, 0.0)
        with self.assertRaisesRegex(AttributeError, "Save Values on the Host"):
            snapshot.count = 1
        values.values["api_address"].text = "127.0.0.1:30060"
        transport.settings.return_value = SettingsListResponse(value=values)
        self.assertIs(context.settings(ConnectionSettings), snapshot)
        self.assertEqual(snapshot.api_address, "127.0.0.1:30059")
        transport.settings.assert_awaited_once()
        await owner._lifecycle("finalize")(object(), transport)
        await owner._lifecycle("initialize")(object(), transport)
        self.assertEqual(context.settings(ConnectionSettings).api_address, "127.0.0.1:30060")
        self.assertIsNot(context.settings(ConnectionSettings), snapshot)
        self.assertEqual(observed, ["127.0.0.1:30059", "127.0.0.1:30060"])
        await owner._lifecycle("finalize")(object(), transport)
        self.assertEqual(released, observed)

    def test_settings_inheritance_overrides_fields_without_duplicate_declarations(self) -> None:
        class SpecializedSettings(ConnectionSettings):
            count: int = Setting.integer("Count", default=2)

        values = harmos.SettingValues(values={name: field.default for name, field in SpecializedSettings._fields().items()})
        self.assertEqual(SpecializedSettings(values).count, 2)
        self.assertEqual(len(SpecializedSettings._fields()), 5)


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
