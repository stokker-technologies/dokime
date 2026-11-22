// Dokime — SPDX-License-Identifier: Apache-2.0
// One Documentation Entry Served Under /<entry>/.

import { fileURLToPath } from "node:url";
import { satteri } from "@astrojs/markdown-satteri";
import starlight from "@astrojs/starlight";
import svelte from "@astrojs/svelte";
import tailwindcss from "@tailwindcss/vite";
import { defineConfig } from "astro/config";
import { d2Plugin } from "./src/d2";

const entry = process.env.DOCS_ENTRY ?? "dev";

export default defineConfig({
  base: `/${entry}/`,
  server: { host: "127.0.0.1", port: 4321 },
  vite: {
    plugins: [tailwindcss()],
    resolve: {
      alias: {
        "@assets": fileURLToPath(new URL("../assets", import.meta.url)),
      },
    },
    server: { strictPort: true },
  },
  markdown: {
    processor: satteri({
      features: { directive: true },
      mdastPlugins: [d2Plugin()],
    }),
  },
  integrations: [
    svelte(),
    starlight({
      title: "Dokime",
      logo: {
        light: "@assets/organization/logos/transparent-on-light.svg",
        dark: "@assets/organization/logos/transparent-on-dark.svg",
        replacesTitle: false,
      },
      customCss: ["./src/styles/custom.css"],
      components: {
        ThemeSelect: "./src/components/ReleaseSelect.astro",
      },
      social: [
        {
          icon: "github",
          label: "Source",
          href: "https://github.com/stokker-technologies/dokime",
        },
      ],
      sidebar: [
        "index",
        {
          label: "Tutorials",
          items: [{ autogenerate: { directory: "tutorials" } }],
        },
        {
          label: "How-to Guides",
          items: [{ autogenerate: { directory: "how-to" } }],
        },
        {
          label: "Reference",
          items: [
            { autogenerate: { directory: "reference" } },
            {
              label: "Rust API",
              link: "https://docs.rs/dokime",
              attrs: { target: "_blank" },
            },
          ],
        },
        {
          label: "Concepts",
          items: [{ autogenerate: { directory: "concepts" } }],
        },
        {
          label: "Project",
          items: [{ autogenerate: { directory: "project" } }],
        },
      ],
    }),
  ],
});
