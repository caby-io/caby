import adapter from '@sveltejs/adapter-node';
import { sveltePreprocess } from 'svelte-preprocess';
import { fileURLToPath } from 'node:url';

const libDir = fileURLToPath(new URL('./src/lib', import.meta.url));

/** @type {import('@sveltejs/kit').Config} */
const config = {
	// Consult https://kit.svelte.dev/docs/integrations#preprocessors
	// for more information about preprocessors
	preprocess: sveltePreprocess({
		scss: {
			importer: (url) => {
				if (!url.startsWith('$lib/')) return null;
				return { file: `${libDir}/${url.slice('$lib/'.length)}` };
			}
		}
	}),

	kit: {
		// adapter-auto only supports some environments, see https://kit.svelte.dev/docs/adapter-auto for a list.
		// If your environment is not supported, or you settled on a specific environment, switch out the adapter.
		// See https://kit.svelte.dev/docs/adapters for more information about adapters.
		adapter: adapter()
	},

	compilerOptions: {
		runes: true
	}
};

export default config;
