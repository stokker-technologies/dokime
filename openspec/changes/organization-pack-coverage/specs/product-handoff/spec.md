## REMOVED Requirements

### Requirement: Isolate Explicit Desktop Storage

**Reason**: Runtime pack import, pending packs and active packs are removed. Desktop application directories continue to respect the explicit application root independently of appearance.
**Migration**: Package the complete organization pack into the installer. Existing application-folder packs are ignored and preserved.
