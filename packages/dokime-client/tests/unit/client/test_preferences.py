# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Settings Operation Round Trip"""

from dokime_client.client import _contracts


def test_settings_operation_round_trip() -> None:
    operation = _contracts.operation("Settings", "UpdateApplicationSettings")
    request = _contracts.request(
        "Settings",
        "UpdateApplicationSettings",
        update={
            "expected_revision": 7,
            "settings": {
                "client": {"connect_timeout_ms": 1234, "request_timeout_ms": 5678},
                "telemetry": {"interval_ms": 42},
            },
        },
    )
    received = operation.request.FromString(request.SerializeToString())
    assert received.update.expected_revision == 7
    assert received.update.settings.client.connect_timeout_ms == 1234
    assert received.update.settings.client.request_timeout_ms == 5678
    assert received.update.settings.telemetry.interval_ms == 42

    response = _contracts.construct(
        operation.response.DESCRIPTOR,
        {
            "revision": 8,
            "settings": received.update.settings,
        },
    )
    restored = operation.response.FromString(response.SerializeToString())
    assert restored.revision == 8
    assert restored.settings == received.update.settings
