// Dokime — SPDX-License-Identifier: Apache-2.0
// One Documentation Entry: the Guides and References of One Source, Served Under /<entry>/.

import { fileURLToPath } from "node:url";
import { unified } from "@astrojs/markdown-remark";
import starlight from "@astrojs/starlight";
import tailwindcss from "@tailwindcss/vite";
import workspace from "../../vite.config.ts";
import { defineConfig } from "astro/config";
import { remarkDiagrams } from "./src/diagrams";
import { remarkGuides } from "./src/guides";

// The Docs Task Selects the Entry and Source Revision; Direct Runs Preview the Development Entry.
const root = fileURLToPath(new URL("../..", import.meta.url));
const entry = process.env.DOCS_ENTRY ?? "dev";
const base = `/${entry}/`;
const source = process.env.DOCS_SOURCE ?? "https://github.com/stokker-technologies/dokime/blob/dev/";

export default defineConfig({
  base,
  server: { host: "127.0.0.1", port: 4321 },
  vite: { resolve: workspace.resolve, plugins: [tailwindcss()], server: { strictPort: true } },
  outDir: `../../target/docs/${entry}`,
  markdown: {
    processor: unified({ remarkPlugins: [remarkGuides({ root, base, source }), remarkDiagrams()] }),
  },
  integrations: [
    starlight({
      title: "Dokime",
      logo: {
        light: "../../organization/logos/transparent-on-light.svg",
        dark: "../../organization/logos/transparent-on-dark.svg",
        replacesTitle: false,
      },
      customCss: ["./src/theme.css"],
      components: { ThemeSelect: "./src/components/ReleaseSelect.astro" },
      social: [{ icon: "github", label: "Source", href: "https://github.com/stokker-technologies/dokime" }],
      sidebar: [
        { label: "Start", items: ["index", "deployment", "support"] },
        { label: "Presentation Examples", link: "/examples/" },
        {
          label: "Guides",
          items: ["gui", "execution-host", "plugins", "plugin-security", "query-api", "screenshots/readme"],
        },
        { label: "SDKs", items: ["rust-sdk", "sdk-python", "sdk-csharp", "sdk-cpp"] },
        { label: "Remote Access", items: [{ autogenerate: { directory: "remote" } }] },
        { label: "Operations Reference", items: [{ autogenerate: { directory: "reference/operations" } }] },
        { label: "Project", items: ["architecture", "tooling", "compliance", "privacy", "contributing", "security"] },
      ],
    }),
  ],
});
