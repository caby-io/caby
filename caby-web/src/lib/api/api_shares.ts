import type { Entry } from '$lib/files/entry';
import type { OverviewEntry } from '$lib/files/overview/overview_entry';
import { ApiRequestBuilder, type ApiClient, type ApiResponse } from './client';

export const CABY_GUEST_TOKEN = 'caby-guest-token';

export type ShareAuthOptions = {
	open: boolean;
	password: boolean;
};

export type GetShareData = {
	id: string;
	space: string;
	root_name: string;
	auth: ShareAuthOptions;
};

export type AuthShareData = {
	permissions: string[];
};

export type ListShareData = {
	path: string;
	parent_dir: string | null;
	entries: Array<Entry>;
};

export type ShareSummary = {
	id: string;
	owner_id: string;
	spec_path: string;
	root_entry: string;
	created_at: string;
	expires_at: string | null;
	expired: boolean;
};

export type ListSpaceSharesData = {
	shares: ShareSummary[];
};

export const listSpaceShares = async (
	client: ApiClient,
	space: string
): Promise<ApiResponse<ListSpaceSharesData>> => {
	const req = ApiRequestBuilder.get(`shares/list/${space}`).intoRequest();
	return await client.exec(req);
};

const withGuest = (client: ApiClient, builder: ApiRequestBuilder): ApiRequestBuilder => {
	if (client.auth.guest_token) {
		builder.addHeaders({ [CABY_GUEST_TOKEN]: client.auth.guest_token });
	}
	return builder.noRedirect();
};

export const getShare = async (
	client: ApiClient,
	id: string
): Promise<ApiResponse<GetShareData>> => {
	const req = withGuest(client, ApiRequestBuilder.get(`shares/${id}`)).intoRequest();
	return await client.exec(req);
};

export const authSharePassword = async (
	client: ApiClient,
	id: string,
	password: string
): Promise<ApiResponse<AuthShareData>> => {
	const req = withGuest(
		client,
		ApiRequestBuilder.post(`shares/${id}/auth/password`).withJsonBody({ password })
	).intoRequest();
	return await client.exec(req);
};

export const listShare = async (
	client: ApiClient,
	id: string,
	path: string
): Promise<ApiResponse<ListShareData>> => {
	const suffix = path ? `/${path}` : '';
	const req = withGuest(client, ApiRequestBuilder.get(`shares/${id}/list${suffix}`)).intoRequest();
	return await client.exec(req);
};

export type ShareOverviewData = {
	entries: OverviewEntry[];
};

export const getShareOverview = async (
	client: ApiClient,
	id: string
): Promise<ApiResponse<ShareOverviewData>> => {
	const req = withGuest(client, ApiRequestBuilder.get(`shares/${id}/overview`)).intoRequest();
	return await client.exec(req);
};
