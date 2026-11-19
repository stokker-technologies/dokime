# Starter Project

Install the project and its tools with `uv sync`. Copy `.env.example` to `.env`
and set `DOKIME_REMOTE` to the desktop's address or exported `.pair` file.

Run `uv run python -m starter_project` to register the plugin, run the capture
procedure and observe temperature every 100 ms, seal its cycle and save evidence
under `_output/YYYY-MM-DD_HH-MM-SS.zip`, using the cycle creation time.
The ZIP contains `report.html`, `manifest.json` and `executions/01 Confirmation/`
with the execution report, readable JSON evidence and Arrow telemetry. Answer the question
in the desktop. The host must support execution sealing and sealed progress.

Run `uv run python -m starter_project.plugin` to serve only the plugin, or
`mise run dev` to start a disposable host. Run `uv run python -m unittest` or
`mise run test` to check the plugin's description without a host.
