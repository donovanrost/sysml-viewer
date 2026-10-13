import { defineConfig } from 'vite';

export default defineConfig({
  clearScreen: false,
  server: {
    // Rust build artifacts must not trigger a browser reload and discard models.
    watch: { ignored: ['**/src-tauri/**', '**/release/**', '**/.build/**', '**/target/**', '**/vendor/**', '**/packages/kernel-wasm/**'] },
  },
});
