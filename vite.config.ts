import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import wasm from "vite-plugin-wasm";

export default defineConfig(({ command }) => ({
  base: command === "build" ? "/SCP-Foundation-editor-web/" : "/",
  plugins: [vue(), wasm()],
}));
