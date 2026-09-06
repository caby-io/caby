import { redirect } from '@sveltejs/kit';
import { client } from '$lib/stores/client.svelte';
import type { LayoutLoad } from './$types';

export const load: LayoutLoad = ({ url }) => {
	if (client.isAuthenticated()) return;

	const target = url.pathname + url.search;
	redirect(302, `/login?redirect=${encodeURIComponent(target)}`);
};
