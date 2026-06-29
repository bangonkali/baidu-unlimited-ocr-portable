import react from '@vitejs/plugin-react';
import { defineConfig } from 'vite';

const apiProxyTarget = process.env.UOCR_DEV_API_PROXY;

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
  plugins: [react()],
  server: apiProxyTarget ? { proxy: { '/api': apiProxyTarget } } : undefined,
});
