import { redirect } from '@sveltejs/kit';
import { client } from '$lib/stores/client.svelte';
import type { LayoutLoad } from './$types';

export const load: LayoutLoad = async ({ url, parent }) => {
	await parent();
	if (!client.isAuthenticated()) return;

	const target = url.searchParams.get('redirect');
	const safe = target && target.startsWith('/') && !target.startsWith('//') ? target : '/files';
	redirect(302, safe);
};
