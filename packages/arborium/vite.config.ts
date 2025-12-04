import { defineConfig, Plugin } from 'vite';
import { resolve } from 'path';
import { copyFileSync, mkdirSync, readdirSync } from 'fs';
import dts from 'vite-plugin-dts';

// Plugin to copy CSS theme files to dist
function copyThemes(): Plugin {
  return {
    name: 'copy-themes',
    closeBundle() {
      const themesDir = resolve(__dirname, 'src/themes');
      const outDir = resolve(__dirname, 'dist/themes');

      mkdirSync(outDir, { recursive: true });

      const files = readdirSync(themesDir);
      for (const file of files) {
        if (file.endsWith('.css')) {
          copyFileSync(resolve(themesDir, file), resolve(outDir, file));
        }
      }
    },
  };
}

export default defineConfig({
  plugins: [
    dts({
      include: ['src/**/*'],
      outDir: 'dist',
    }),
    copyThemes(),
  ],
  build: {
    lib: {
      entry: {
        arborium: resolve(__dirname, 'src/index.ts'),
        'arborium.iife': resolve(__dirname, 'src/iife.ts'),
      },
      formats: ['es'],
      fileName: (_format, entryName) => {
        if (entryName === 'arborium.iife') {
          return 'arborium.iife.js';
        }
        return `${entryName}.js`;
      },
    },
    rollupOptions: {
      output: {
        // For the IIFE entry, we want it self-contained
        inlineDynamicImports: false,
      },
    },
    target: 'es2022',
    minify: 'esbuild',
    sourcemap: true,
  },
});
