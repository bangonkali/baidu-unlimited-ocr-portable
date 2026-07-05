import { tanstackRouter } from '@tanstack/router-plugin/vite';
import react from '@vitejs/plugin-react';
import { defineConfig } from 'vite';

const apiProxyTarget = process.env.TRAPO_DEV_API_PROXY;

export default defineConfig({
  build: {
    rollupOptions: {
      output: {
        manualChunks(id) {
          if (id.includes('/node_modules/react')) {
            return 'react';
          }
          return undefined;
        },
      },
    },
  },
  plugins: [tanstackRouter({ autoCodeSplitting: true, target: 'react' }), react()],
  server: apiProxyTarget
    ? {
        proxy: {
          '/api': {
            changeOrigin: true,
            target: apiProxyTarget,
            ws: true,
          },
        },
      }
    : undefined,
});
