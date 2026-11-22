## Decisions

The owner has agreed all eight decisions in this change. Proposed public shapes:

```python
log = logging.getLogger(__name__)

class ToolStatus(Observer[StatusRow]):
    interval = timedelta(milliseconds=200)
    async def sample(self, ctx: Context) -> StatusRow: ...
    async def close(self, ctx: Context) -> None: ...

class Burst(WindowObserver[BurstColumns]):
    async def window(self, ctx: Context, window: Window) -> BurstColumns: ...

class Events(Publisher[EventRow]):
    async def publish(self, ctx: Context, out: object) -> None: ...

class MyPlugin(Plugin):
    api_address = Setting.text("Remote API Address", default="127.0.0.1:30059", required=True)
    dll_timeout = Setting.duration("DLL Host Timeout", default=timedelta(seconds=5), min=timedelta(milliseconds=100), max=timedelta(seconds=30))
    api_token = Secret("Remote API Token")
    remote = Resource(open=open_remote_api, close=close_remote_api)
    disconnected = Record(str, label="Tool Disconnected")
    observers = (ToolStatus, Burst, Events)

class Connectivity(TestTemplate):
    wifi_channel = Parameter.integer("Wi-Fi Channel", choices=(1, 6, 11))
    uses_streams = (ToolStatus,)
    async def run(self, ctx: Context) -> None:
        channel = ctx.parameter(Connectivity.wifi_channel)
        timeout = ctx.setting(MyPlugin.dll_timeout)
        token = await ctx.secret(MyPlugin.api_token)
        remote = await ctx.resource(MyPlugin.remote)
        log.debug("Connected")
        result = await ctx.operation(OP)
```

## Ownership and Replacement

Operation invocation has one private transport implementation shared by Context and Client. Remove BoundOperation and migrate all owned callers. Preserve transport contract fingerprint checks and actionable failures.

Settings constraints belong to host save admission; template constraints belong to host case creation. Python must not substitute local checks for missing host capabilities. Typed declarations replace SettingDeclaration/SettingValue and untyped parameter helpers. Plugin descriptors derive identifiers with __set_name__; records and streams are qualified by plugin identity.

The logging handler owns a context variable, ordered buffers and flush recursion protection. Every awaited host call must pass that buffer boundary; handler completion flushes even when author code raises. Observer records require host/library support for broadcasting to capturing executions. Sampling instance lifetime must follow host start/stop, never inferred from elapsed time.

Resources belong to activation and are released even if finalize fails. Settings snapshots refresh on activation/Retry/restart. OperationPolicy.transient is a local exception predicate; only wire-representable policy fields are sent remotely. No per-plugin failure mapper.

Record.field and Record.time_since replace string-based record expressions. Keep the existing reference_band/reference_line API when parameter references already read directly. Observer plots are placed explicitly; no automatic sections.

## Proof and Delivery

Author and strictly validate this change before source edits. Commit by decision group, close implementation checkboxes only with focused proof, then run the applicable final gates once. Root task arguments follow the current task implementation. Generated output must come from generate tasks. Documentation remains owner-managed; report affected statements. Retire with the specification finish lifecycle only after every delivery obligation is met.
