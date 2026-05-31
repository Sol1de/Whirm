import { defineConfig } from 'vitest/config'
import tailwindcss from '@tailwindcss/vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'
import path from "path";

const aliases = {
  "@components": path.resolve("./frontend/src/components"),
  "@ui":         path.resolve("./frontend/src/components/ui"),
  "@pages":      path.resolve("./frontend/src/pages"),
  "@services":   path.resolve("./frontend/src/services"),
  "@stores":     path.resolve("./frontend/src/stores"),
  "@types":      path.resolve("./frontend/src/types"),
  "@utils":      path.resolve("./frontend/src/utils"),
  "@lib":        path.resolve("./frontend/src/lib"),
};

// https://vite.dev/config/
export default defineConfig({
  plugins: [
    svelte(),
    tailwindcss()
  ],
  resolve: { alias: aliases },
  server: {
    watch: {
      ignored: ['**/backend/target/**'],
    },
  },
  test: {
    environment: 'jsdom',
    include: ['frontend/src/test/**/*.test.ts'],
    globals: true,
    setupFiles: ['frontend/src/test/setup.ts'],
    alias: aliases,
  },
})
