# @generated from protocol descriptors; do not edit.
from dokime_sdk._wire.host.v1.secrets_pb2 import SecretsListResponse
from dokime_sdk._wire.host.v1.settings_pb2 import SettingsGetResponse, SettingsSetResponse

from . import media, plots, plugins

SecretsListResponse.__module__ = "dokime_sdk.client"
SecretsListResponse.__qualname__ = "host.SecretsListResponse"
SettingsGetResponse.__module__ = "dokime_sdk.client"
SettingsGetResponse.__qualname__ = "host.SettingsGetResponse"
SettingsSetResponse.__module__ = "dokime_sdk.client"
SettingsSetResponse.__qualname__ = "host.SettingsSetResponse"

__all__ = ["SecretsListResponse", "SettingsGetResponse", "SettingsSetResponse", "media", "plots", "plugins"]
