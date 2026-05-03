import { defineConfig } from 'vite'
import tailwindcss from '@tailwindcss/vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'
import path from "path";

// https://vite.dev/config/
export default defineConfig({
  plugins: [
    svelte(),
    tailwindcss()
  ],
  resolve: {
    alias: {
      "@components": path.resolve("./frontend/src/components"),
      "@ui":         path.resolve("./frontend/src/components/ui"),
      "@pages":      path.resolve("./frontend/src/pages"),
      "@services":   path.resolve("./frontend/src/services"),
      "@stores":     path.resolve("./frontend/src/stores"),
      "@types":      path.resolve("./frontend/src/types"),
      "@utils":      path.resolve("./frontend/src/utils"),
      "@hooks":      path.resolve("./frontend/src/hooks"),
    },
  },
})
