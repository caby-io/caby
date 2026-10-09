import { SvelteSet } from 'svelte/reactivity';
import { getFilesOverview } from '$lib/api/api_files';
import { ApiStatus, type ApiClient } from '$lib/api/client';
import type { OverviewEntry } from '$lib/files/overview/overview_entry';
import { join } from '$lib/fs';

export const overview = $state<{ space: string | null; entries: OverviewEntry[] }>({
	space: null,
	entries: []
});

export const expanded_dirs = new SvelteSet<string>();

let latest_space: string | null = null;

export const tryRefreshOverview = async (client: ApiClient, space: string) => {
	latest_space = space;
	const resp = await getFilesOverview(client, space, '', true);
	if (latest_space !== space) return;
	if (resp.status !== ApiStatus.StatusSuccess || !resp.data) return;
	overview.space = space;
	overview.entries = resp.data.entries;
};

export const expandToPath = (href_base: string, path: string) => {
	const segments = path.split('/').filter(Boolean);
	for (let i = 1; i < segments.length; i++) {
		expanded_dirs.add(join(href_base, ...segments.slice(0, i)));
	}
};
