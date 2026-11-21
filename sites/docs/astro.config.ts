// Dokime — SPDX-License-Identifier: Apache-2.0
// One Documentation Entry: the Guides and References of One Source, Served Under /<entry>/.

import { fileURLToPath } from "node:url";
import { unified } from "@astrojs/markdown-remark";
import starlight from "@astrojs/starlight";
import tailwindcss from "@tailwindcss/vite";
import { defineConfig } from "astro/config";
import workspace from "../../vite.config.ts";
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
      components: {
        ThemeProvider: "./src/components/ThemeProvider.astro",
        ThemeSelect: "./src/components/ReleaseSelect.astro",
      },
      expressiveCode: {
        customizeTheme(theme) {
          theme.fg = "var(--foreground)";
          theme.colors["editor.foreground"] = "var(--foreground)";
          theme.settings = [
            { scope: ["comment", "punctuation.definition.comment"], settings: { foreground: "var(--muted-foreground)" } },
            { scope: ["keyword", "storage", "constant.language"], settings: { foreground: "var(--link)" } },
            { scope: ["string", "entity.name", "support", "constant.numeric"], settings: { foreground: "var(--foreground)" } },
          ];
          return theme;
        },
        styleOverrides: {
          borderRadius: "var(--brand-radius)",
          borderColor: "var(--border)",
          codeBackground: "var(--muted)",
          codeForeground: "var(--foreground)",
          codeSelectionBackground: "var(--input)",
          frames: {
            editorBackground: "var(--muted)",
            terminalBackground: "var(--muted)",
            editorActiveTabBackground: "var(--muted)",
            editorActiveTabForeground: "var(--foreground)",
            editorTabBarBackground: "var(--background)",
            terminalTitlebarBackground: "var(--background)",
            terminalTitlebarForeground: "var(--foreground)",
            inlineButtonForeground: "var(--foreground)",
            inlineButtonBorder: "var(--input)",
            frameBoxShadowCssValue: "none",
          },
        },
      },
      social: [{ icon: "github", label: "Source", href: "https://github.com/stokker-technologies/dokime" }],
      sidebar: [
        { label: "Start", items: ["index", "deployment", "support"] },
        { label: "Presentation Examples", link: "/examples/" },
        // Autogeneration Matches the Loader's File Paths Relative to This Site.
        { label: "Guides", items: [{ autogenerate: { directory: "../../docs/guides" } }] },
        { label: "SDKs", items: [{ autogenerate: { directory: "../../docs/sdks" } }] },
        { label: "Remote Access", items: [{ autogenerate: { directory: "../../docs/remote" } }] },
        { label: "Operations Reference", items: [{ autogenerate: { directory: "../../target/docs/generated/reference/operations" } }] },
        { label: "Project", items: [{ autogenerate: { directory: "../../docs/project" } }, "contributing", "security"] },
      ],
    }),
  ],
});
