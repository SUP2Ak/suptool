import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// @ts-ignore: process is a nodejs global
import process from "node:process";

const host = process.env.TAURI_DEV_HOST;

// https://vitejs.dev/config/

// @ts-ignore: eslint-disable-next-line require-await
export default defineConfig(async () => {
  // @ts-ignore: eslint-disable-next-line require-await
  await asyncFunction(); // Ajoutez l'appel à votre fonction asynchrone ici
  return {
    plugins: [react()],

    // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
    //
    // 1. prevent vite from obscuring rust errors
    clearScreen: false,
    // 2. tauri expects a fixed port, fail if that port is not available
    server: {
      port: 1420,
      strictPort: true,
      host: host || false,
      hmr: host
        ? {
          protocol: "ws",
          host,
          port: 1421,
        }
        : undefined,
      watch: {
        // 3. tell vite to ignore watching `src-tauri`
        ignored: ["**/src-tauri/**"],
      },
    },
  };
});
