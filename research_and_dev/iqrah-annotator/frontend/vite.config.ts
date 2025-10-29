import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

// Ensures headers are set for ALL served assets (dev + preview)
function coepHeadersPlugin() {
  return {
    name: 'coep-headers',
    configureServer(server: any) {
      server.middlewares.use((req: any, res: any, next: any) => {
        res.setHeader('Cross-Origin-Opener-Policy', 'same-origin')
        res.setHeader('Cross-Origin-Embedder-Policy', 'require-corp')
        res.setHeader('Cross-Origin-Resource-Policy', 'same-origin')
        next()
      })
    },
    configurePreviewServer(server: any) {
      server.middlewares.use((req: any, res: any, next: any) => {
        res.setHeader('Cross-Origin-Opener-Policy', 'same-origin')
        res.setHeader('Cross-Origin-Embedder-Policy', 'require-corp')
        res.setHeader('Cross-Origin-Resource-Policy', 'same-origin')
        next()
      })
    }
  }
}

// https://vite.dev/config/
export default defineConfig({
  plugins: [react(), coepHeadersPlugin()],
  server: {
    port: 5173,
    hmr: {
      // Disable error overlay to avoid COEP iframe warning
      // Errors will still appear in browser console
      overlay: false,
    },
  },
  preview: {
    port: 5174,
  },
  optimizeDeps: {
    // avoid prebundling loaders; we control the URLs ourselves
    exclude: ['@ffmpeg/ffmpeg', '@ffmpeg/util']
  }
})
