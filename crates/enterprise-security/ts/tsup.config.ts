import { defineConfig } from 'tsup';

export default defineConfig({
  entry: {
    index: 'src/index.ts',
    types: 'src/types/index.ts',
    auth: 'src/auth/index.ts',
    authz: 'src/authz/index.ts',
    encryption: 'src/encryption/index.ts',
    compliance: 'src/compliance/index.ts',
    components: 'src/components/index.ts',
    services: 'src/services/index.ts',
  },
  format: ['cjs', 'esm'],
  dts: true,
  splitting: false,
  sourcemap: true,
  clean: true,
  treeshake: true,
  minify: false,
  external: ['react', 'react-dom'],
});
