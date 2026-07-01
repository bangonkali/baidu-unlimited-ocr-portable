import { defineConfig } from 'orval';

export default defineConfig({
  trapo: {
    input: {
      target: '../trapo-server/openapi/trapo.openapi.json',
    },
    output: {
      target: './src/generated/trapo.ts',
      schemas: './src/generated/model',
      client: 'fetch',
      clean: true,
    },
  },
});
