# @generated from protocol descriptors; do not edit.
from dokime._wire.host.v1.secrets_pb2 import SecretsListResponse

from . import media, plots, plugins

SecretsListResponse.__module__ = "dokime.client"
SecretsListResponse.__qualname__ = "host.SecretsListResponse"
from dokime._wire.host.v1.settings_pb2 import SettingsGetResponse

SettingsGetResponse.__module__ = "dokime.client"
SettingsGetResponse.__qualname__ = "host.SettingsGetResponse"
from dokime._wire.host.v1.settings_pb2 import SettingsSetResponse

SettingsSetResponse.__module__ = "dokime.client"
SettingsSetResponse.__qualname__ = "host.SettingsSetResponse"

__all__ = ["SecretsListResponse", "SettingsGetResponse", "SettingsSetResponse", "media", "plots", "plugins"]
