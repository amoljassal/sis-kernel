import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import { VitePWA } from 'vite-plugin-pwa'
import path from 'path'

export default defineConfig({
  plugins: [
    react(),
    VitePWA({
      registerType: 'autoUpdate',
      includeAssets: ['favicon.ico', 'apple-touch-icon.png', 'masked-icon.svg'],
      manifest: {
        name: 'SIS Hybrid AI-Lab',
        short_name: 'SIS AI-Lab',
        description: 'Hardware-Software Co-Design Platform with Natural Language Interface',
        theme_color: '#1f2937',
        background_color: '#111827',
        display: 'standalone',
        start_url: '/',
        scope: '/',
        icons: [
          {
            src: 'pwa-192x192.png',
            sizes: '192x192',
            type: 'image/png'
          },
          {
            src: 'pwa-512x512.png',
            sizes: '512x512',
            type: 'image/png'
          }
        ]
      },
      workbox: {
        globPatterns: ['**/*.{js,css,html,ico,png,svg,wasm}'],
        maximumFileSizeToCacheInBytes: 50 * 1024 * 1024, // 50MB for WASM files
      }
    })
  ],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  server: {
    port: 3000,
    open: true,
    fs: {
      // Allow serving files from the kernel directory for WASM
      allow: ['..']
    }
  },
  build: {
    outDir: 'dist',
    rollupOptions: {
      output: {
        manualChunks: {
          vendor: ['react', 'react-dom'],
          editor: ['monaco-editor', '@monaco-editor/react'],
          three: ['three', '@react-three/fiber', '@react-three/drei'],
          collaboration: ['yjs', 'y-webrtc'],
        }
      }
    },
    target: 'esnext',
    minify: 'terser',
  },
  optimizeDeps: {
    exclude: ['@vite/client', '@vite/env']
  },
  define: {
    'process.env.NODE_ENV': JSON.stringify(process.env.NODE_ENV || 'development')
  }
})