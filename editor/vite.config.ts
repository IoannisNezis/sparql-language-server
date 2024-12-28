import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
import importMetaUrlPlugin from '@codingame/esbuild-import-meta-url-plugin'

export default defineConfig({
	optimizeDeps: {
		esbuildOptions: {
			plugins: [importMetaUrlPlugin]
		}
	},
	resolve: {
		dedupe: ['monaco-editor', 'vscode']
	},
	plugins: [sveltekit()]
});
