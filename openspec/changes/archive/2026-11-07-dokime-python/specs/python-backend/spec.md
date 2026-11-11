## ADDED Requirements

### Requirement: Python-Started Backend
Python SHALL start a backend with a port and configuration through explicit start(), using the
server crate, and stop it in order through close(), without requiring a context manager.

#### Scenario: Normal Exit
- **WHEN** the caller closes the backend with no cycle running
- **THEN** listeners and plugins stop and the data root is released

#### Scenario: Port Unavailable
- **WHEN** the requested port is in use
- **THEN** start raises a typed error naming the address, and no backend remains

### Requirement: Shared Data Root
A Python-started backend SHALL default to the data root the GUI and server use, and accept
an explicit root for distributions with their own.

#### Scenario: Root Already Owned
- **WHEN** the GUI's local backend or another process owns the root
- **THEN** startup raises `BackendRunning` naming the root and leaves its data untouched

#### Scenario: GUI Local Start During a Script
- **WHEN** the GUI starts its local backend while a script owns the root
- **THEN** the GUI reports the existing running-backend refusal and still allows attaching

### Requirement: Pairing Export
A running backend SHALL export a pairing file the GUI can import.

#### Scenario: GUI Attaches
- **WHEN** the GUI imports the exported file while the script runs
- **THEN** it shows the installed and Python plugins, live cycles and their plots

#### Scenario: Export While Not Accepting Calls
- **WHEN** export is called after shutdown begins
- **THEN** it raises a typed error and writes no file

### Requirement: Installed Plugins Load
A Python-started backend SHALL load the root's installed plugins exactly as the GUI's
backend does, with no option to skip them.

#### Scenario: Installed and Python Plugins Together
- **WHEN** the root has an installed plugin and the script registers a Python plugin
- **THEN** both appear in the catalog and either can run

### Requirement: Ephemeral Python Plugins
A Python plugin SHALL exist only in the backend that registered it, while its evidence
remains in the data root.

#### Scenario: GUI Opened Afterwards
- **WHEN** the GUI starts on the root after the script exits
- **THEN** the plugin is absent from its catalog and its sealed evidence stays readable

#### Scenario: Re-Run Without the Plugin
- **WHEN** a user selects a finished execution whose Python plugin is absent
- **THEN** the view reads its evidence and starting it again reports the plugin is unavailable

### Requirement: Script Exit During Work
Unfinished work SHALL follow the existing restart recovery when its script ends.

#### Scenario: Interrupted Script
- **WHEN** a script ends or crashes while an execution runs
- **THEN** the next backend on that root marks it Interrupted and settles its cycle
  without needing the plugin
