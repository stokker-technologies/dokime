# Plugins

Plugins serve typed catalogs and operations through the library SDK. Managed
placement supervises an executable; remote placement connects to a user-owned URL.
Use the [Plugin Guide](../../../../docs/plugins.md) for SDK and bundle commands,
and [Plugin Security](../../../../docs/plugin-security.md) for trust boundaries.
Lifecycle declarations are host-only Protected operations. Context calls inherit
invocation scope and require admitted descriptors/fingerprints. Do not introduce
client bindings for plugin lifecycle operations or assume all context calls are
already implemented; the combined operation-grammar tasks track missing capabilities.
