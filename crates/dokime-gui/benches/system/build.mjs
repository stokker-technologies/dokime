// Dokime — SPDX-License-Identifier: Apache-2.0
import { build } from "vite";
import { fileURLToPath } from "node:url";
import { resolve } from "node:path";
import { copyFile } from "node:fs/promises";

const root = fileURLToPath(new URL("../../", import.meta.url));
const selected = new Map([
  ["/plots/live-frames.ts", new Map([["applyDocumentPlotMessages", "retained_update"]])],
  ["/lib/api/native-plots.ts", new Map([["mapNativePlotWatchPage", "native_frame_adaptation"]])],
]);

// Instrument The Existing Owners Only In The Bench Bundle; Shipping Source Is Unchanged.
await build({
  root,
  resolve: { alias: { "react-dom/client": "react-dom/profiling" } },
  plugins: [
    {
      name: "desktop-bench-observation",
      enforce: "pre",
      transform(source, id) {
        const path = id.replaceAll("\\", "/");
        const functions = [...selected].find(([suffix]) => path.endsWith(suffix))?.[1];
        if (functions) {
          for (const [name, stage] of functions) {
            const declaration = `export function ${name}(`;
            if (!source.includes(declaration)) throw new Error(`Missing benchmark instrumentation owner: ${id}/${name}`);
            source = source.replace(declaration, `function bench_${name}(`);
            source += `\nexport function ${name}(...args: Parameters<typeof bench_${name}>): ReturnType<typeof bench_${name}> {
            const started = performance.now();
            try { return bench_${name}(...args); }
            finally { globalThis.__desktopBench?.sample('${stage}', performance.now() - started); }
          }`;
          }
        }
        if (path.endsWith("/lib/rpc/ipc.ts")) {
          for (const owner of [
            "fromBinary,",
            "frames.push(new Uint8Array(message));",
            "const payload = await call.next();",
            "const frame = frames.shift();",
          ]) {
            if (!source.includes(owner)) throw new Error(`Missing benchmark IPC instrumentation owner: ${owner}`);
          }
          source = source.replace("fromBinary,", "fromBinary as decodeBinary,");
          source += `\nconst fromBinary = (...args: Parameters<typeof decodeBinary>) => {
          const started = performance.now();
          try { return decodeBinary(...args); }
          finally { globalThis.__desktopBench?.sample('protobuf_decode', performance.now() - started); }
        };`;
          source = source.replace(
            'const reply = await invokeOwned<ArrayBuffer>(session, "rpc_stream_open",',
            'const __opening = performance.now();\n  const reply = await invokeOwned<ArrayBuffer>(session, "rpc_stream_open",',
          );
          source = source.replace(
            "const opened = responseBytes(session, reply, 16);",
            `if (path === '/dokime.client.v1.plots.Snapshots/Watch') globalThis.__desktopBench?.sample('client_open', performance.now() - __opening);
            const opened = responseBytes(session, reply, 16);`,
          );
          source = source.replace(
            "frames.push(new Uint8Array(message));",
            `frames.push(new Uint8Array(message));
          if (path === '/dokime.client.v1.plots.Snapshots/Watch') globalThis.__desktopBench?.arrival(message.byteLength, frames.length);`,
          );
          source = source.replace(
            "const payload = await call.next();",
            `const __received = performance.now();
          const payload = await call.next();
          if (path === '/dokime.client.v1.plots.Snapshots/Watch') globalThis.__desktopBench?.sample('transport_bridge_wait', performance.now() - __received);`,
          );
          source = source.replace(
            "const frame = frames.shift();",
            `if (path === '/dokime.client.v1.plots.Snapshots/Watch' && globalThis.__desktopBench?.stallUntil > performance.now())
          await new Promise(resolve => setTimeout(resolve, globalThis.__desktopBench.stallUntil - performance.now()));
        const frame = frames.shift();`,
          );
        }
        if (path.endsWith("/components/document/PlotGroupGrid.tsx")) {
          if (!source.includes('data-live-mounted="true"')) throw new Error("Observe Plot Drawing: Restore Its Mounted Card Owner");
          source = source.replace('data-live-mounted="true"', 'data-live-mounted="true" data-bench-has-payload={Boolean(state.payload)}');
          for (const owner of ["<VisiblePlotCard", "viewportMode={viewportMode}\n    />"]) {
            if (!source.includes(owner)) throw new Error(`Missing benchmark card instrumentation owner: ${owner}`);
          }
          source = source.replace(
            "<VisiblePlotCard",
            '<div key={descriptor.plot_id} data-bench-plot={descriptor.plot_id} data-bench-error={liveStates[descriptor.plot_id]?.status === "error" || Boolean(liveStates[descriptor.plot_id]?.failure && !liveStates[descriptor.plot_id]?.payload)} data-bench-ready={Boolean(liveStates[descriptor.plot_id]?.payload)} data-bench-revision={globalThis.__desktopBench.frame(liveStates[descriptor.plot_id]?.payload)}><VisiblePlotCard',
          );
          source = source.replace("viewportMode={viewportMode}\n    />", "viewportMode={viewportMode}\n    /></div>");
        }
        return { code: source, map: null };
      },
    },
  ],
  build: {
    outDir: resolve(process.argv[2]),
    rolldownOptions: { input: resolve(root, "benches/system/index.html") },
  },
});
await copyFile(resolve(process.argv[2], "benches/system/index.html"), resolve(process.argv[2], "index.html"));
