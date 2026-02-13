import path from "path"
import react from "@vitejs/plugin-react"
import { defineConfig } from "vite"

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
  build: {
    outDir: "dist",
    emptyOutDir: true,
    // Generate assets that can be served from any base path
    assetsDir: "assets",
  },
  server: {
    proxy: {
      "/api/v1": {
        target: "http://localhost:3000",
        changeOrigin: true,
      },
      "/health": {
        target: "http://localhost:3000",
        changeOrigin: true,
      },
      "/swagger-ui": {
        target: "http://localhost:3000",
        changeOrigin: true,
      },
      "/api-docs": {
        target: "http://localhost:3000",
        changeOrigin: true,
      },
    },
  },
})
