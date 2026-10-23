# Native Desktop Test Parity

## Decisions

The user requires Windows local validation to include the desktop UI stories. Linux and Windows execute one shared ordinary journey inventory; platform branches exist only for native driver/runtime adaptation and genuinely platform-specific installer verification. The earlier Windows deferral is replaced.

The Dokime owner owns authored desktop bootstrap, mise/stkr configuration, harness, README/testing guidance, combined validation, and the next signed PR update. The parent owns the provider completion guards, transaction ordering, and focused state regression test. The Hub owner owns any canonical CI generator change. Writers use the existing integration worktree with disjoint file ownership.

Windows bootstrap detects the actual WebView2 version, obtains its matching Microsoft driver, verifies that executable's version, and exposes it through the job environment. Local and hosted runners use the same bootstrap. No machine-specific installed driver is assumed.

The harness keeps native pointer interaction, existing assertions and deadlines. Scrolling and layout settling must precede interaction; failure evidence must distinguish a missed click from a dialog disappearing afterward. The settings-dialog cause remains an inference until native execution confirms it.

A provider group with pending work is not a new group to dispatch. Stop/shutdown/terminal guards precede replay acceptance in the transaction; completion must settle remaining work instead of repeatedly submitting a cancelled invocation. Ordinary valid transaction replay remains idempotent.

Windows helper processes remain hidden and attached to the owned harness; Linux retains process groups for cleanup. The actual GUI remains visible.

Microsoft EdgeDriver encodes valued application arguments as one `--name=value` entry. The Windows adapter supplies `--endpoint=<value>`; WebKitGTK retains its native argument vector. Both adapters execute the same journeys.

Default owned startup follows the application's automatic local-backend selection. The walkthrough verifies owned status, listener availability and drain on close; it does not require a manual connection button before the connected shell.

Generated starter subprocesses resolve their declared tools independently of machine and system mise configuration. Both MISE_GLOBAL_CONFIG_FILE and MISE_SYSTEM_CONFIG_FILE select one real empty normal-temp file kept alive for the child process. MISE_CEILING_PATHS bounds ancestor discovery at the generated repository so parent project configuration cannot reintroduce machine tools. Installed data/cache homes and TEMP/TMP remain unchanged. An offline resolution regression supplies hostile private-tool declarations and verifies they cannot enter the starter's tool set.

## Validation

Run focused checks during implementation. After every implementation item closes, run the newly applicable Windows UI proof and affected build/check/test lanes on the combined source, retaining prior unaffected full proof. Required native Linux and Windows CI remain integration gates. No retry, timeout, assertion, or required-check weakening is permitted.
