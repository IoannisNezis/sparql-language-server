import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
import importMetaUrlPlugin from '@codingame/esbuild-import-meta-url-plugin'
import wasm from "vite-plugin-wasm";
import topLevelAwait from "vite-plugin-top-level-await";

export default defineConfig({
	optimizeDeps: {
		esbuildOptions: {
			plugins: [importMetaUrlPlugin]
		}
	},
	resolve: {
		dedupe: ['monaco-editor', 'vscode']
	},
	plugins: [sveltekit(), wasm(), topLevelAwait()]
});
