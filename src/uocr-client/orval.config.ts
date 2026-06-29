import { defineConfig } from 'orval';

export default defineConfig({
  uocr: {
    input: {
      target: '../uocr-server/openapi/uocr.openapi.json',
    },
    output: {
      target: './src/generated/uocr.ts',
      schemas: './src/generated/model',
      client: 'fetch',
      clean: true,
    },
  },
});
