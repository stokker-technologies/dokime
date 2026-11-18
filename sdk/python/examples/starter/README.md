# Starter Project

Install the project and its tools with `uv sync`. Copy `.env.example` to `.env`
and set `DOKIME_REMOTE` to the desktop's address or exported `.pair` file.

Run `uv run python -m starter_project` to register the plugin, run the capture
procedure, seal its cycle and save evidence under `_output/`. Answer the question
in the desktop. The host must support execution sealing and sealed progress.

Run `uv run python -m starter_project.plugin` to serve only the plugin, or
`mise run dev` to start a disposable host. Run `uv run python -m unittest` or
`mise run test` to check the plugin's description without a host.
