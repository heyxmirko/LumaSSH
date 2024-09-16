import { defineConfig } from 'vite'
import { resolve } from 'path'
import { copy } from 'vite-plugin-copy'

export default defineConfig({
  clearScreen: false,
  server: {
    strictPort: true,
  },
  envPrefix: ['VITE_', 'TAURI_PLATFORM', 'TAURI_ARCH', 'TAURI_FAMILY', 'TAURI_PLATFORM_VERSION', 'TAURI_PLATFORM_TYPE', 'TAURI_DEBUG'],
  build: {
    target: process.env.TAURI_PLATFORM == 'windows' ? 'chrome105' : 'safari13',
    minify: !process.env.TAURI_DEBUG ? 'esbuild' : false,
    sourcemap: !!process.env.TAURI_DEBUG,
    rollupOptions: {
      input: {
        main: resolve(__dirname, 'index.html'), // Make sure your entry file is correct
        terminal: resolve(__dirname, 'terminal.html'),
      }
    }
  },
  plugins: [
    copy({
      targets: [
        { src: 'terminal.js', dest: 'dist' },
        { src: 'assets/*', dest: 'dist/assets' }
      ],
      hook: 'writeBundle',
    })
  ]
})
