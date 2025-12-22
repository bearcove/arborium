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
      beforeWriteFile: (filePath, content) => ({
        content,
        filePath: filePath.endsWith('index.d.ts') ? filePath.replace('index.d.ts', 'arborium.d.ts') : filePath,
      }),
    }),
    copyThemes(),
  ],
  build: {
    lib: {
      entry: resolve(__dirname, 'src/index.ts'),
      formats: ['es'],
      fileName: () => 'arborium.js',
    },
    target: 'es2022',
    minify: 'esbuild',
    sourcemap: true,
  },
});
