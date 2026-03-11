/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  theme: {
    extend: {
      colors: {
        canvas: "#0d1117",
        surface: "#161b22",
        elevated: "#21262d",
        border: "#30363d",
        accent: "#58a6ff",
        "accent-muted": "#1f3a5f",
        success: "#3fb950",
        warning: "#d29922",
        danger: "#f85149",
      },
      fontFamily: {
        mono: ["JetBrains Mono", "Fira Code", "Consolas", "monospace"],
      },
    },
  },
  plugins: [],
};
