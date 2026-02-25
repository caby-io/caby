import type { Entry } from '$lib/files/entry';
import type { OverviewEntry } from '$lib/files/overview/overview_entry';
import { ApiClient, ApiRequestBuilder, type ApiResponse } from './client';

export type ListFilesResp = {
	path: string | null;
	parent_dir: string | null;
	current_dir: string;
	entries: Array<Entry>;
};

export const listFiles = async (
	client: ApiClient,
	space: string,
	path: string
): Promise<ApiResponse<ListFilesResp>> => {
	const req = ApiRequestBuilder.get(`files/list/${space}/${path}`).intoRequest();
	return await client.exec(req);
};

export type FilesOverviewResp = {
	path: string | null;
	parent_dir: string | null;
	current_dir: string;
	entries: Array<OverviewEntry>;
};

export const filesOverview = async (
	client: ApiClient,
	space: string,
	path: string
): Promise<ApiResponse<FilesOverviewResp>> => {
	const req = ApiRequestBuilder.get(`files/overview/${space}/${path}`).intoRequest();
	return await client.exec(req);
};
