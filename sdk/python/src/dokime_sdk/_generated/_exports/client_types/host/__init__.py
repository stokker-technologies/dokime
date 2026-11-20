# @generated from protocol descriptors; do not edit.
from dokime_sdk._wire.host.v1.plugins_pb2 import (
    PLUGIN_STATE_CONNECTING,
    PLUGIN_STATE_READY,
    PLUGIN_STATE_STOPPED,
    PLUGIN_STATE_STOPPING,
    PLUGIN_STATE_UNAVAILABLE,
    PLUGIN_STATE_UNSPECIFIED,
    PluginChange,
    PluginDescription,
    PluginEntry,
    PluginGuest,
    PluginManaged,
    PluginPlacement,
    PluginProcess,
    PluginRemote,
    PluginsCreateResponse,
    PluginsDeleteResponse,
    PluginsGetResponse,
    PluginsListResponse,
    PluginSnapshot,
    PluginsReplaceResponse,
    PluginsRestartResponse,
    PluginsSetResponse,
    PluginState,
    PluginsWatchResponse,
    PluginValue,
)
from dokime_sdk._wire.host.v1.secrets_pb2 import SecretsListResponse
from dokime_sdk._wire.host.v1.settings_pb2 import SettingsGetResponse, SettingsSetResponse
from dokime_sdk._wire.host.v1.status_pb2 import StatusGetResponse, StatusWatchResponse

from . import media, plots, plugins

PluginChange.__module__ = "dokime_sdk.client"
PluginChange.__qualname__ = "host.PluginChange"
PluginDescription.__module__ = "dokime_sdk.client"
PluginDescription.__qualname__ = "host.PluginDescription"
PluginEntry.__module__ = "dokime_sdk.client"
PluginEntry.__qualname__ = "host.PluginEntry"
PluginGuest.__module__ = "dokime_sdk.client"
PluginGuest.__qualname__ = "host.PluginGuest"
PluginManaged.__module__ = "dokime_sdk.client"
PluginManaged.__qualname__ = "host.PluginManaged"
PluginPlacement.__module__ = "dokime_sdk.client"
PluginPlacement.__qualname__ = "host.PluginPlacement"
PluginProcess.__module__ = "dokime_sdk.client"
PluginProcess.__qualname__ = "host.PluginProcess"
PluginRemote.__module__ = "dokime_sdk.client"
PluginRemote.__qualname__ = "host.PluginRemote"
PluginSnapshot.__module__ = "dokime_sdk.client"
PluginSnapshot.__qualname__ = "host.PluginSnapshot"
PluginValue.__module__ = "dokime_sdk.client"
PluginValue.__qualname__ = "host.PluginValue"
PluginsCreateResponse.__module__ = "dokime_sdk.client"
PluginsCreateResponse.__qualname__ = "host.PluginsCreateResponse"
PluginsDeleteResponse.__module__ = "dokime_sdk.client"
PluginsDeleteResponse.__qualname__ = "host.PluginsDeleteResponse"
PluginsGetResponse.__module__ = "dokime_sdk.client"
PluginsGetResponse.__qualname__ = "host.PluginsGetResponse"
PluginsListResponse.__module__ = "dokime_sdk.client"
PluginsListResponse.__qualname__ = "host.PluginsListResponse"
PluginsReplaceResponse.__module__ = "dokime_sdk.client"
PluginsReplaceResponse.__qualname__ = "host.PluginsReplaceResponse"
PluginsRestartResponse.__module__ = "dokime_sdk.client"
PluginsRestartResponse.__qualname__ = "host.PluginsRestartResponse"
PluginsSetResponse.__module__ = "dokime_sdk.client"
PluginsSetResponse.__qualname__ = "host.PluginsSetResponse"
PluginsWatchResponse.__module__ = "dokime_sdk.client"
PluginsWatchResponse.__qualname__ = "host.PluginsWatchResponse"
SecretsListResponse.__module__ = "dokime_sdk.client"
SecretsListResponse.__qualname__ = "host.SecretsListResponse"
SettingsGetResponse.__module__ = "dokime_sdk.client"
SettingsGetResponse.__qualname__ = "host.SettingsGetResponse"
SettingsSetResponse.__module__ = "dokime_sdk.client"
SettingsSetResponse.__qualname__ = "host.SettingsSetResponse"
StatusGetResponse.__module__ = "dokime_sdk.client"
StatusGetResponse.__qualname__ = "host.StatusGetResponse"
StatusWatchResponse.__module__ = "dokime_sdk.client"
StatusWatchResponse.__qualname__ = "host.StatusWatchResponse"

__all__ = [
    "PLUGIN_STATE_CONNECTING",
    "PLUGIN_STATE_READY",
    "PLUGIN_STATE_STOPPED",
    "PLUGIN_STATE_STOPPING",
    "PLUGIN_STATE_UNAVAILABLE",
    "PLUGIN_STATE_UNSPECIFIED",
    "PluginChange",
    "PluginDescription",
    "PluginEntry",
    "PluginGuest",
    "PluginManaged",
    "PluginPlacement",
    "PluginProcess",
    "PluginRemote",
    "PluginSnapshot",
    "PluginState",
    "PluginValue",
    "PluginsCreateResponse",
    "PluginsDeleteResponse",
    "PluginsGetResponse",
    "PluginsListResponse",
    "PluginsReplaceResponse",
    "PluginsRestartResponse",
    "PluginsSetResponse",
    "PluginsWatchResponse",
    "SecretsListResponse",
    "SettingsGetResponse",
    "SettingsSetResponse",
    "StatusGetResponse",
    "StatusWatchResponse",
    "media",
    "plots",
    "plugins",
]
