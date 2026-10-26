import { defineConfig } from "vitepress";

const base = process.env.DOCS_BASE ?? "/live-dmx/";

export default defineConfig({
  lang: "en-US",
  title: "Live DMX",
  description:
    "Documentation for Live DMX, a fast lighting control application for live performance.",
  base,
  lastUpdated: true,
  themeConfig: {
    nav: [
      { text: "Manual", link: "/manual/" },
      { text: "Contributing", link: "/contributing/" },
    ],
    sidebar: {
      "/manual/": [
        {
          text: "Manual",
          items: [{ text: "Overview", link: "/manual/" }],
        },
        {
          text: "Setup",
          items: [
            { text: "Patch", link: "/manual/setup/patch" },
            { text: "Configure", link: "/manual/setup/configure" },
            { text: "Input", link: "/manual/setup/input" },
            { text: "Output", link: "/manual/setup/output" },
          ],
        },
        {
          text: "Live",
          items: [
            { text: "Intensity", link: "/manual/live/intensity" },
            { text: "Colors", link: "/manual/live/colors" },
          ],
        },
        {
          text: "Reference",
          items: [
            { text: "Navigation", link: "/manual/reference/navigation" },
            { text: "Glossary", link: "/manual/reference/glossary" },
          ],
        },
      ],
      "/contributing/": [
        {
          text: "Contributing",
          items: [{ text: "Overview", link: "/contributing/" }],
        },
        {
          text: "Architecture",
          items: [
            { text: "Backend", link: "/contributing/architecture" },
            { text: "Frontend", link: "/contributing/frontend" },
            { text: "DMX output", link: "/contributing/dmx-output" },
            { text: "Rendering", link: "/contributing/rendering" },
            { text: "Fixtures", link: "/contributing/fixtures" },
          ],
        },
        {
          text: "Workflow",
          items: [
            { text: "Show openers", link: "/contributing/show-openers" },
            { text: "Releasing", link: "/contributing/releasing" },
          ],
        },
      ],
    },
    search: {
      provider: "local",
    },
    editLink: {
      pattern: "https://github.com/baptiste-may/live-dmx/edit/main/docs/:path",
      text: "Edit this page",
    },
    socialLinks: [
      {
        icon: "github",
        link: "https://github.com/baptiste-may/live-dmx",
      },
    ],
  },
});
