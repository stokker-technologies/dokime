## REMOVED Requirements

### Requirement: Python-Started Backend

**Reason**: Python is a pure remote SDK; runtime hosting belongs to Dokime.
**Migration**: Serve a Python plugin and connect to an independently started host.

### Requirement: Shared Data Root

**Reason**: Python no longer opens application storage.
**Migration**: Select storage when starting Dokime and use a pairing file from Python.

### Requirement: Pairing Export

**Reason**: The Python-owned backend is removed.
**Migration**: Export pairing through the running Dokime host.

### Requirement: Installed Plugins Load

**Reason**: Installed plugin lifecycle belongs to runtime, not Python.
**Migration**: Start Dokime normally; add the script's remote URL through plugin operations.

### Requirement: Ephemeral Python Plugins

**Reason**: Remote entries persist until removed; availability is an entry state.
**Migration**: Explicitly add/remove URLs; frozen execution evidence remains readable offline.

### Requirement: Script Exit During Work

**Reason**: Scripts no longer own a backend or its restart recovery.
**Migration**: Plugin process changes interrupt their live runs under host-executions recovery.
