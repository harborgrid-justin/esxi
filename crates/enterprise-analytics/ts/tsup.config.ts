import { defineConfig } from 'tsup';

export default defineConfig({
  entry: {
    index: 'src/index.ts',
    types: 'src/types/index.ts',
    query: 'src/query/index.ts',
    visualization: 'src/visualization/index.ts',
    components: 'src/components/index.ts',
    services: 'src/services/index.ts',
  },
  format: ['cjs', 'esm'],
  dts: true,
  splitting: false,
  sourcemap: true,
  clean: true,
  external: ['react', 'react-dom', 'd3'],
  treeshake: true,
  minify: false,
  target: 'es2022',
});
