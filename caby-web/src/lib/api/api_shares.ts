import type { Entry } from '$lib/files/entry';
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

const withGuest = (client: ApiClient, builder: ApiRequestBuilder): ApiRequestBuilder => {
	if (client.auth.guest_token) {
		builder.addHeaders({ [CABY_GUEST_TOKEN]: client.auth.guest_token });
	}
	return builder.noRedirect();
};

export const getShare = async (
	client: ApiClient,
	space: string,
	id: string
): Promise<ApiResponse<GetShareData>> => {
	const req = withGuest(client, ApiRequestBuilder.get(`shares/${space}/${id}`)).intoRequest();
	return await client.exec(req);
};

export const authSharePassword = async (
	client: ApiClient,
	space: string,
	id: string,
	password: string
): Promise<ApiResponse<AuthShareData>> => {
	const req = withGuest(
		client,
		ApiRequestBuilder.post(`shares/${space}/${id}/auth/password`).withJsonBody({ password })
	).intoRequest();
	return await client.exec(req);
};

export const listShare = async (
	client: ApiClient,
	space: string,
	id: string,
	path: string
): Promise<ApiResponse<ListShareData>> => {
	const suffix = path ? `/${path}` : '';
	const req = withGuest(
		client,
		ApiRequestBuilder.get(`shares/${space}/${id}/list${suffix}`)
	).intoRequest();
	return await client.exec(req);
};
