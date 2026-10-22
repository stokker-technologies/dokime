# Dokime Desktop Validation

Use the native WebView for desktop IPC and process-ownership coverage. Browser mocks do not establish native transport behavior; Linux evidence does not establish Windows behavior.

## Live Native Desktop

Exercise the built shell and server with the shipped plugin in an isolated application storage root. Cover template selection, parameters, prompts, live evidence, reports and shutdown.

Verify that closing an owned backend settles its executions and closing an attached shell leaves the external server alive.

## Visual And Diagnostic Pass

Cover loading, empty, failed, running, completed and disconnected states in the mounted workflows. Exercise disconnect, reconnect and backend switching so an old connection cannot update the new view.
