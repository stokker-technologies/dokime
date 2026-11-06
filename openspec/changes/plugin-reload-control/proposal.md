## Why
Saving plugin settings currently restarts programs automatically. Operators need explicit control over when saved values take effect and consistent Reload actions.

## What Changes
- Launch configuration seeds unsaved fields; operator saves override those seeds.
- Save settings without restarting; compare saved values against the running generation in the backend and expose changed field identifiers.
- Replace plugin Retry with Reload for installed and configured programs; preserve restart admission, pending and refusal behavior.
- Stop and restart every observer with its program, including plugin-activated observers, using newly prepared settings.
- Align table statuses in their own column; expose Settings, Reload and Delete directly; provide Reload on every plugin page.
- Update affected guides and generated contracts; remove obsolete restart/Retry description wording.

## Impact
Runtime plugin lifecycle and settings, operation contracts/server/client bindings, GUI settings and existing deterministic tests. README claims remain valid. No other repository changes. Owner request agrees the scope and scenarios. App/view restarts and attached GUIs read backend state; a backend restart starts a new generation with saved settings and clears differences.
