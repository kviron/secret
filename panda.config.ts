import { defineConfig } from "@pandacss/dev";

export default defineConfig({
  preflight: true,

  include: ["./src/**/*.{js,jsx,ts,tsx}"],

  exclude: [],

  theme: {
    extend: {
      tokens: {
        colors: {
          bg: {
            primary: { value: "#0d1117" },
            secondary: { value: "#161b22" },
            tertiary: { value: "#21262d" },
          },
          border: {
            default: { value: "#30363d" },
            hover: { value: "#484f58" },
          },
          text: {
            primary: { value: "#e6edf3" },
            secondary: { value: "#8b949e" },
            muted: { value: "#6e7681" },
          },
          accent: {
            default: { value: "#ff3b30" },
            hover: { value: "#ff5c50" },
          },
          success: { value: "#3fb950" },
          warning: { value: "#d29922" },
          danger: { value: "#f85149" },
        },
        fonts: {
          heading: { value: "'Inter', system-ui, sans-serif" },
          body: { value: "'Inter', system-ui, sans-serif" },
          mono: { value: "'JetBrains Mono', 'Fira Code', monospace" },
        },
        radii: {
          sm: { value: "4px" },
          md: { value: "8px" },
          lg: { value: "12px" },
          xl: { value: "16px" },
        },
        spacing: {
          xs: { value: "4px" },
          sm: { value: "8px" },
          md: { value: "16px" },
          lg: { value: "24px" },
          xl: { value: "32px" },
          "2xl": { value: "48px" },
        },
      },
    },
  },

  outdir: "styled-system",

  jsxFramework: "solid",
});
