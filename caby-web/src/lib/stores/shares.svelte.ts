import { listSpaceShares, type ShareSummary } from '$lib/api/api_shares';
import { ApiStatus, type ApiClient } from '$lib/api/client';

export const shares = $state<{ space: string | null; list: ShareSummary[] }>({
	space: null,
	list: []
});

let latest_space: string | null = null;

export const tryRefreshShares = async (client: ApiClient, space: string) => {
	latest_space = space;
	const resp = await listSpaceShares(client, space);
	if (latest_space !== space) return;
	if (resp.status !== ApiStatus.StatusSuccess || !resp.data) return;
	shares.space = space;
	shares.list = resp.data.shares;
};
