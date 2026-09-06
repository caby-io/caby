import { client } from '$lib/stores/client.svelte';
import type { Token } from '$lib/api/client';
import type { LayoutLoad } from './$types';

export const ssr = false;

export const load: LayoutLoad = async () => {
	const login_token = await cookieStore.get('login_token');
	if (!login_token?.value) return;

	const token: Token = JSON.parse(decodeURIComponent(login_token.value));
	client.setLoginToken(token);
};
