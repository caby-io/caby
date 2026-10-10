import { getSpaces } from '$lib/api/api_spaces';
import { ApiStatus, type ApiClient } from '$lib/api/client';
import type { Space } from '$lib/space';

export const spaces = $state<{ list: Space[] }>({ list: [] });

export const tryLoadSpaces = async (client: ApiClient) => {
	const resp = await getSpaces(client);
	if (resp.status === ApiStatus.StatusSuccess && resp.data) spaces.list = resp.data;
};
