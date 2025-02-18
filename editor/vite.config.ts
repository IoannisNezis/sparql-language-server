import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
import importMetaUrlPlugin from '@codingame/esbuild-import-meta-url-plugin'
import wasm from "vite-plugin-wasm";
import topLevelAwait from "vite-plugin-top-level-await";

export default defineConfig(({ mode }) => ({
	optimizeDeps: {
		esbuildOptions: {
			plugins: [importMetaUrlPlugin]
		},
		include: [
			'@testing-library/react',
			'vscode/localExtensionHost',
			'vscode-textmate',
			'vscode-oniguruma'
		]
	},
	worker: {
		format: "es",
		plugins: () => [
			wasm(),
			topLevelAwait()
		]
	},
	resolve: {
		alias:
		{
			"qlue-ls": mode === "development" ? "qlue-ls-dev" : "qlue-ls",
		},
	},
	plugins: [sveltekit(), wasm(), topLevelAwait()]
}));
